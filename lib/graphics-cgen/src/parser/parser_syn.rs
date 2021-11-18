use anyhow::{anyhow, Context, Result};
use log::trace;
use relative_path::RelativePath;

use std::path::{Path, PathBuf};
use syn::{File, Item, ItemMod, ItemStruct};

use crate::{
    builder::StructBuilder,
    model::{CGenType, Model},
};

pub fn from_syn(file_path: &Path) -> anyhow::Result<Model> {
    assert!(file_path.is_absolute());

    let mut model = Model::new();

    // let cur_dir = file_path.parent()?;

    process_syn_model(&mut model, &file_path, true)?;

    Ok(model)
}

fn process_syn_model(model: &mut Model, file_path: &Path, is_root: bool) -> Result<()> {
    assert!(file_path.is_absolute());

    trace!("Parsing model in {}", file_path.display());

    let ast = load_syn_file(file_path)?;

    for item in &ast.items {
        match item {
            Item::Mod(e) => {
                process_syn_mod(model, file_path, e, is_root)
                    .context(format!("Cannot parse mod from {}", file_path.display()))?;
            }
            Item::Struct(e) => {
                assert!(e.attrs.len() == 0);

                process_syn_struct(model, e).context(format!(
                    "Cannot parse struct from '{}'",
                    file_path.display()
                ))?;
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
    file_path: &Path,
    item_mod: &ItemMod,
    is_root: bool,
) -> anyhow::Result<()> {
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

    process_syn_model(model, &final_path, false)
}

fn process_syn_struct(model: &mut Model, syn_struct: &ItemStruct) -> anyhow::Result<()> {
    let struct_name = syn_struct.ident.to_string();
    trace!("Parsing struct {}", &struct_name);
    let mut builder = StructBuilder::new(model, &struct_name);

    for mb in &syn_struct.fields {
        let membername = mb.ident.as_ref().unwrap().to_string();
        let typename = {
            match &mb.ty {
                syn::Type::Path(e) => {
                    assert!(e.path.segments.len() == 1);
                    e.path.segments[0].ident.to_string()
                }
                _ => {
                    panic!("Unmanged type");
                }
            }
        };

        builder = builder.add_member(&membername, &typename)?;
    }
    let product = builder.build()?;

    model.add(CGenType::Struct(product))?;

    Ok(())
}

fn load_syn_file(file_path: &Path) -> Result<File> {
    let file_content = std::fs::read_to_string(&file_path)
        .context(format!("Failed to load {}", file_path.display()))?;
    let ast = syn::parse_file(&file_content)
        .context(format!("Failed to parse {}", file_path.display()))?;
    Ok(ast)
}
