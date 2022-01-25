use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use lgn_tracing::{trace, warn};
use relative_path::RelativePath;
use syn::{File, Item, ItemMod, ItemStruct, Lit, NestedMeta};

use super::ParsingResult;
use crate::{
    builder::{DescriptorSetBuilder, PipelineLayoutBuilder, StructBuilder},
    db::{self, CGenType, Model},
};

pub(crate) fn from_syn(file_path: &Path) -> Result<ParsingResult> {
    assert!(file_path.is_absolute());

    proc_macro2::fallback::force();

    let mut input_dependencies = Vec::new();
    let mut model = db::create();

    process_syn_model(&mut model, &mut input_dependencies, file_path, true)?;

    Ok(ParsingResult {
        input_dependencies,
        model,
    })
}

fn process_syn_model(
    model: &mut Model,
    input_dependencies: &mut Vec<PathBuf>,
    file_path: &Path,
    is_root: bool,
) -> Result<()> {
    assert!(file_path.is_absolute());

    trace!("Parsing model in {}", file_path.display());

    let ast = load_syn_file(file_path)?;

    input_dependencies.push(file_path.to_owned());

    for item in &ast.items {
        match item {
            Item::Mod(e) => {
                process_syn_mod(model, input_dependencies, file_path, e, is_root)
                    .context(format!("Cannot parse mod from {}", file_path.display()))?;
            }
            Item::Struct(e) => {
                let property_bag = PropertyBag::from(e)?;
                let property_bag_ty = property_bag.ty.as_str();
                match property_bag_ty {
                    "Struct" => process_syn_struct(model, &property_bag).context(format!(
                        "Cannot parse struct from '{}'",
                        file_path.display()
                    ))?,
                    "DescriptorSet" => process_syn_descriptorset(model, &property_bag).context(
                        format!("Cannot parse descriptorset from '{}'", file_path.display()),
                    )?,
                    "PipelineLayout" => process_syn_pipelinelayout(model, &property_bag).context(
                        format!("Cannot parse descriptorset from '{}'", file_path.display()),
                    )?,
                    _ => {
                        warn!("Unknown type {} ", property_bag_ty);
                    }
                };
            }
            _ => {
                panic!();
            }
        }
    }

    Ok(())
}

fn process_syn_mod(
    model: &mut Model,
    input_dependencies: &mut Vec<PathBuf>,
    file_path: &Path,
    item_mod: &ItemMod,
    is_root: bool,
) -> Result<()> {
    let mod_name = item_mod.ident.to_string();
    let file_folder = file_path.parent().unwrap();
    trace!("Parsing mod {}", &mod_name);
    let mut final_path = PathBuf::new();
    let is_mod = file_path.file_name().unwrap().eq("mod.cgen");

    if is_root || is_mod {
        let rel_path = RelativePath::new(&mod_name);
        let mut rel_path_with_ext = rel_path.to_relative_path_buf();
        rel_path_with_ext.set_extension("cgen");
        let abs_path = rel_path_with_ext.to_logical_path(file_folder);
        if abs_path.exists() {
            final_path = abs_path;
        }
    }

    if !final_path.has_root() {
        let rel_path = RelativePath::new(&mod_name);
        let mut rel_path_with_ext = rel_path.to_relative_path_buf();
        rel_path_with_ext.push("mod.cgen");
        let abs_path = rel_path_with_ext.to_logical_path(file_folder);
        if abs_path.exists() {
            final_path = abs_path;
        }
    }

    if !final_path.has_root() {
        return Err(anyhow!(
            "Cannot resolve mod {} in file {}",
            mod_name,
            file_path.display()
        ));
    }

    process_syn_model(model, input_dependencies, &final_path, false)
}

#[allow(clippy::todo)]
fn expect<T>(lit: &syn::Lit) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let result = match lit {
        Lit::Str(_)
        | Lit::ByteStr(_)
        | Lit::Byte(_)
        | Lit::Bool(_)
        | Lit::Verbatim(_)
        | Lit::Char(_) => todo!(),
        Lit::Int(e) => e.base10_parse::<T>(),
        Lit::Float(e) => e.base10_parse::<T>(),
    };
    result.map_err(|err| anyhow!(err))
}

#[derive(Default)]
struct Attribute {
    name: String,
    value: Option<syn::Lit>,
}

impl Attribute {
    fn from_name_value(name_value: &syn::MetaNameValue) -> Result<Self> {
        let name = name_value
            .path
            .get_ident()
            .ok_or(anyhow!("Invalid atttribute definition #tofo"))?
            .to_string();

        let value = &name_value.lit;
        Ok(Self {
            name,
            value: Some(value.clone()),
        })
    }

    fn from_path(path: &syn::Path) -> Result<Self> {
        let name = path
            .get_ident()
            .ok_or(anyhow!("Invalid atttribute definition #tofo"))?
            .to_string();
        Ok(Self { name, value: None })
    }

    #[allow(clippy::todo)]
    fn value<T>(&self) -> Result<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        if let Some(lit) = &self.value {
            expect(lit)
        } else {
            Err(anyhow!("#todo"))
        }
    }
}

#[derive(Default)]
struct Attributes {
    attribs: Vec<Attribute>,
}

impl Attributes {
    fn from_attrib(attr: &syn::Attribute) -> Result<Self> {
        let mut attribs = Vec::new();
        if !attr.tokens.is_empty() {
            let meta = attr.parse_meta()?;
            match &meta {
                syn::Meta::List(meta_list) => {
                    for nested_meta in &meta_list.nested {
                        match nested_meta {
                            NestedMeta::Meta(meta) => {
                                match meta {
                                    syn::Meta::NameValue(nv) => {
                                        attribs.push(Attribute::from_name_value(nv)?);
                                    }
                                    syn::Meta::Path(p) => {
                                        attribs.push(Attribute::from_path(p)?);
                                    }
                                    syn::Meta::List(_) => {
                                        return Err(anyhow!("Invalid atttribute definition #tofo"));
                                    }
                                };
                            }
                            NestedMeta::Lit(_) => {}
                        }
                    }
                }
                _ => return Err(anyhow!("Invalid atttribute definition #tofo")),
            };
        }
        Ok(Self { attribs })
    }

    #[allow(clippy::todo)]
    fn expect<T>(&self, name: &str) -> Result<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        let attrib = self
            .attribs
            .iter()
            .find(|x| x.name == name)
            .ok_or(anyhow!("#todo"))?;
        attrib.value()
    }
}

#[derive(Default)]
struct PropertyType {
    name: String,
    tpl: Option<String>,
    array_len: Option<u32>,
}

impl PropertyType {
    fn is_templated(&self) -> bool {
        self.tpl != None
    }

    fn template_name(&self) -> &str {
        assert!(self.is_templated());
        if let Some(name) = &self.tpl {
            return name.as_str();
        }
        panic!()
    }

    fn is_array(&self) -> bool {
        self.array_len != None
    }
}

#[derive(Default)]
struct Property {
    name: String,
    ty: PropertyType,
    _attribs: Attributes,
}

impl Property {
    #[allow(clippy::todo)]
    fn from_field(field: &syn::Field) -> Result<Self> {
        //
        // Field format:
        //
        // #[attrib_list] (optionnal)
        // field_name: field_type
        //
        // attrib: name | name = value
        // attrib_list: attrib_list | attrib
        // simple_type: ident
        // templated_type: simple_type<simple_type>
        // type_def: simple_type | templated_type
        // field_type: type_def | [type_def; N]
        // field_name: ident
        //

        let field_name = field
            .ident
            .as_ref()
            .ok_or(anyhow!("Invalid field definition"))?;
        let field_name = field_name.to_string();

        //
        // Extract property type
        //
        let field_type = match &field.ty {
            syn::Type::Path(p) => {
                let type_def = parse_type_def(&p.path)?;
                PropertyType {
                    name: type_def.0,
                    tpl: type_def.1,
                    array_len: None,
                }
            }
            syn::Type::Array(e) => {
                let mut prop_ty = match e.elem.as_ref() {
                    syn::Type::Path(p) => {
                        let type_def = parse_type_def(&p.path)?;
                        PropertyType {
                            name: type_def.0,
                            tpl: type_def.1,
                            array_len: None,
                        }
                    }
                    _ => return Err(anyhow!("#todo")),
                };
                prop_ty.array_len = match &e.len {
                    syn::Expr::Lit(lit) => Some(expect::<u32>(&lit.lit)?),
                    _ => return Err(anyhow!("#todo")),
                };
                prop_ty
            }
            _ => {
                panic!("Unmanged type");
            }
        };

        //
        // Parse attributes
        //
        if field.attrs.is_empty() {
            Ok(Self {
                name: field_name,
                ty: field_type,
                ..Self::default()
            })
        } else {
            if field.attrs.len() != 1 {
                return Err(anyhow!("Invalid format #todo"));
            }
            let attr = &field.attrs[0];
            Ok(Self {
                name: field_name,
                ty: field_type,
                _attribs: Attributes::from_attrib(attr)?,
            })
        }
    }
}

fn parse_type_def(path: &syn::Path) -> Result<(String, Option<String>)> {
    if path.segments.len() != 1 {
        return Err(anyhow!("Invalid field type"));
    }
    let path_seg = &path.segments[0];
    let type_name = path_seg.ident.to_string();
    let mut tpl_name = None;
    match &path_seg.arguments {
        syn::PathArguments::None => (),
        syn::PathArguments::AngleBracketed(e) => {
            if e.args.len() != 1 {
                return Err(anyhow!("Invalid template argument"));
            }
            let tpl_arg = &e.args[0];
            if let syn::GenericArgument::Type(syn::Type::Path(tpl_path)) = tpl_arg {
                let path = &tpl_path.path;
                if path.segments.len() != 1 {
                    return Err(anyhow!("Invalid template type"));
                }
                let segment = &path.segments[0];
                tpl_name = Some(segment.ident.to_string());
            } else {
                return Err(anyhow!("#todo"));
            }
        }
        syn::PathArguments::Parenthesized(_) => return Err(anyhow!("#todo")),
    }

    Ok((type_name, tpl_name))
}

struct PropertyBag {
    nam: String,
    ty: String,
    attribs: Attributes,
    props: Vec<Property>,
}

impl PropertyBag {
    fn from(item: &ItemStruct) -> Result<Self> {
        let name = item.ident.to_string();
        let attr = if !item.attrs.is_empty() {
            if item.attrs.len() != 1 {
                return Err(anyhow!("Invalid format #todo"));
            }
            Some(&item.attrs[0])
        } else {
            None
        };
        let ty = if let Some(attr) = attr {
            attr.path
                .get_ident()
                .ok_or(anyhow!("Invalid format #todo"))?
                .to_string()
        } else {
            "Struct".to_string()
        };
        let attribs = if let Some(attr) = attr {
            Attributes::from_attrib(attr)?
        } else {
            Attributes::default()
        };

        //
        // Extract properties
        //
        let props: Result<Vec<_>> = item.fields.iter().map(Property::from_field).collect();
        let props = props?;

        Ok(Self {
            nam: name,
            ty,
            attribs,
            props,
        })
    }
}

fn process_syn_struct(model: &mut Model, prop_bag: &PropertyBag) -> Result<()> {
    trace!("Parsing struct {}", &prop_bag.nam);

    let name = prop_bag.nam.as_str();
    let mut builder = StructBuilder::new(model, name);

    for prop in &prop_bag.props {
        if prop.ty.is_templated() {
            return Err(anyhow!("Invalid struct member definition"));
        }
        builder =
            builder.add_member(prop.name.as_str(), prop.ty.name.as_str(), prop.ty.array_len)?;
    }

    let struct_type = builder.build()?;
    model.add(name, CGenType::Struct(struct_type))?;

    Ok(())
}

#[allow(clippy::todo)]
fn process_syn_descriptorset(model: &mut Model, prop_bag: &PropertyBag) -> Result<()> {
    trace!("Parsing descriptorset {}", &prop_bag.nam);

    let name = prop_bag.nam.as_str();
    let frequency = prop_bag.attribs.expect("frequency")?;
    let mut builder = DescriptorSetBuilder::new(model, name, frequency);

    for prop in &prop_bag.props {
        let prop_name = prop.name.as_str();
        let prop_ty_name = prop.ty.name.as_str();
        match prop_ty_name {
            "Sampler" => {
                if prop.ty.is_templated() {
                    return Err(anyhow!("Invalid descriptor definition"));
                }
                builder = builder.add_samplers(prop_name, prop.ty.array_len)?;
            }
            "ConstantBuffer" => {
                if !prop.ty.is_templated() || prop.ty.is_array() {
                    return Err(anyhow!("Invalid descriptor definition"));
                }
                builder = builder.add_constant_buffer(prop_name, prop.ty.template_name())?;
            }
            "StructuredBuffer" | "RWStructuredBuffer" => {
                if !prop.ty.is_templated() {
                    return Err(anyhow!("Invalid descriptor definition"));
                }
                let read_write = prop_ty_name.starts_with("RW");

                builder = builder.add_structured_buffer(
                    prop_name,
                    prop.ty.array_len,
                    prop.ty.template_name(),
                    read_write,
                )?;
            }
            "ByteAddressBuffer" | "RWByteAddressBuffer" => {
                if prop.ty.is_templated() {
                    return Err(anyhow!("Invalid descriptor definition"));
                }
                let read_write = prop_ty_name.starts_with("RW");

                builder =
                    builder.add_byte_address_buffer(prop_name, prop.ty.array_len, read_write)?;
            }
            "Texture2D" | "RWTexture2D" | "Texture3D" | "RWTexture3D" | "Texture2DArray"
            | "RWTexture2DArray" | "TextureCube" | "TextureCubeArray" => {
                if !prop.ty.is_templated() {
                    return Err(anyhow!("Invalid descriptor definition"));
                }
                let read_write = prop_ty_name.starts_with("RW");
                let sub_ty_pos = read_write as usize * "RW".len() + "Texture".len();
                let sub_ty = &prop_ty_name[sub_ty_pos..];

                builder = builder.add_texture(
                    prop_name,
                    sub_ty,
                    prop.ty.template_name(),
                    prop.ty.array_len,
                    read_write,
                )?;
            }
            _ => {
                todo!("{} not implemented", prop_ty_name);
            }
        }
    }

    let descriptor_set = builder.build()?;
    model.add(name, descriptor_set)?;

    Ok(())
}

#[allow(clippy::todo)]
fn process_syn_pipelinelayout(model: &mut Model, prop_bag: &PropertyBag) -> Result<()> {
    trace!("Parsing pipelinelayout {}", &prop_bag.nam);

    let name = prop_bag.nam.as_str();
    let mut builder = PipelineLayoutBuilder::new(model, name);

    for prop in &prop_bag.props {
        match prop.ty.name.as_str() {
            "DescriptorSet" => {
                if !prop.ty.is_templated() {
                    return Err(anyhow!("#todo"));
                }
                builder =
                    builder.add_descriptor_set(prop.name.as_str(), prop.ty.template_name())?;
            }
            "PushConstant" => {
                if !prop.ty.is_templated() {
                    return Err(anyhow!("#todo"));
                }
                builder = builder.add_push_constant(prop.name.as_str(), prop.ty.template_name())?;
            }
            _ => {
                todo!();
            }
        }
    }

    let pipeline_layout = builder.build()?;
    model.add(name, pipeline_layout)?;

    Ok(())
}

fn load_syn_file(file_path: &Path) -> Result<File> {
    let file_content = std::fs::read_to_string(&file_path)
        .context(format!("Failed to load {}", file_path.display()))?;
    let ast = syn::parse_file(&file_content)
        .context(format!("Failed to parse {}", file_path.display()))?;
    Ok(ast)
}
