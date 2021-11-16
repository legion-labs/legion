use anyhow::{Context, Result};
use log::trace;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use syn::{File, Item, ItemMod, ItemStruct};

use crate::{
    builder::StructBuilder,
    model::{CGenType, Model},
};

struct SynMod {
    file_name: String,
}

impl SynMod {
    fn new(item: &ItemMod) -> Self {
        let file_name = item.ident.to_string();
        Self { file_name }
    }
}

pub fn from_syn(file_path: &Path) -> anyhow::Result<Arc<Model>> {
    let mut model = Model::new();

    process_syn_model(&mut model, &file_path)?;

    Ok(Arc::new(model))
}

fn process_syn_model(model: &mut Model, file_path: &Path) -> Result<()> {
    assert!(file_path.is_absolute());

    let file_folder = file_path.parent().unwrap();
    let ast = load_syn_file(file_path)?;

    for item in &ast.items {
        match item {
            Item::Mod(e) => {                
                let syn_mod = SynMod::new(e);
                process_syn_mod(model, file_folder, &syn_mod).context(format!(
                    "Cannot include file '{}' from {}",
                    syn_mod.file_name,
                    file_path.display()
                ))?;                
            }
            Item::Struct(e) => {
                assert!(e.attrs.len() == 0);

                process_syn_struct(model, e).context(format!(
                    "Cannot add Struct '{}' from '{}'",
                    "todo",
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

fn process_syn_mod(model: &mut Model, file_folder: &Path, syn_mod: &SynMod) -> anyhow::Result<()> {
    let mut inc_path = PathBuf::from_str(&syn_mod.file_name)?;
    if inc_path.is_relative() {
        let mut abs_path = PathBuf::from(file_folder);
        abs_path.push(inc_path);
        inc_path = abs_path;
    }

    process_syn_model(model, &inc_path)?;

    Ok(())
}

fn process_syn_struct(model: &mut Model, syn_struct: &ItemStruct) -> anyhow::Result<()> {

    let struct_name = syn_struct.ident.to_string();

    trace!("START: process_syn_struct {}", &struct_name );

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

        trace!( "add member {} : {}", &membername, &typename );

        builder = builder.add_member(&membername, &typename)?;
    }
    let product = builder.build()?;

    model.add(CGenType::Struct(product))?;

    let x = model.object_iter::<CGenType>().unwrap();
    for i in x {
        dbg!(&i );
    }


    trace!("END" );

    Ok(())
}

fn load_syn_file(file_path: &Path) -> Result<File> {
    let file_content = std::fs::read_to_string(&file_path)
        .context(format!("Failed to load {}", file_path.display()))?;
    let ast = syn::parse_file(&file_content)
        .context(format!("Failed to parse {}", file_path.display()))?;
    Ok(ast)
}
