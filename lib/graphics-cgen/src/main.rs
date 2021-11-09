use anyhow::Result;
use graphics_cgen::{model::*, parser::*};
use std::{
    io::Write,
    path::{Path, PathBuf},
    str::FromStr
};

fn main() {
    let mut file_path = PathBuf::new();
    file_path.push(
        "D:\\git\\github.com\\legion-labs\\legion\\lib\\graphics-cgen\\data\\data_test2.yaml",
    );

    match run(&file_path) {
        Ok(_) => {}
        Err(e) => {
            for i in e.chain() {
                eprintln!("{}", i);
            }
        }
    }
    
    test::test1();
}

struct CGenContext<'cgen> {
    output_folder: PathBuf,
    model: &'cgen Model,
}

#[derive(Debug)]
struct Line {
    indent: u32,
    content: String,
}

struct FileWriter {
    lines: Vec<Line>,
    indent: u32,
}

impl FileWriter {
    fn new() -> Self {
        FileWriter {
            lines: Vec::new(),
            indent: 0,
        }
    }

    fn new_line(&mut self) {
        self.add_line("".to_owned());
    }

    fn add_line(&mut self, line: String) {
        self.lines.push(Line {
            indent: self.indent,
            content: line,
        });
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn unindent(&mut self) {
        assert!(self.indent > 0);
        self.indent -= 1;
    }

    fn to_string(&self) -> String {
        let mut result = String::new();

        for line in &self.lines {
            for _ in 0..line.indent {
                result.push('\t');
            }
            result.push_str(&line.content);
            result.push('\n');
        }

        result
    }
}

fn get_hlsl_typestring<'a>(cgen_type: &'a CGenType) -> &'a str {
    let typestring = match &cgen_type {
        CGenType::Float1 => "float",
        CGenType::Float2 => "float2",
        CGenType::Float3 => "float3",
        CGenType::Float4 => "float4",
        CGenType::Complex(s) => s.as_str(),
    };

    typestring
}

fn get_member_declaration(member: &StructMember) -> String {
    let typestring = get_hlsl_typestring(&member.cgen_type);

    format!("{} {};", typestring, member.name)
}

fn generate_hlsl_struct(context: &CGenContext, struct_name: &str) -> Product {
    let def = context.model.structs().get(struct_name).unwrap();
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!("#ifndef TYPE_{}", def.name.to_uppercase()));
    writer.add_line(format!("#define TYPE_{}", def.name.to_uppercase()));
    writer.new_line();

    writer.indent();

    // dependencies
    let deps = context
        .model
        .get_struct_type_dependencies(struct_name)
        .unwrap();

    // let deps: Vec<_> = def
    //     .members
    //     .iter()
    //     .map(|m| &m.cgen_type)
    //     .filter(|t| {
    //         if let CGenType::Complex(_) = &t {
    //             true
    //         } else {
    //             false
    //         }
    //     })
    //     .collect();

    if !deps.is_empty() {
        for m in deps {
            writer.add_line(format!("#include \"{}.hlsl\"", m.to_string()));
        }
        writer.new_line();
    }

    // struct
    writer.add_line(format!("struct {} {{", def.name));

    writer.indent();
    for m in &def.members {
        writer.add_line(get_member_declaration(m));
    }
    writer.unindent();

    writer.add_line(format!("}}; // {}", def.name));

    writer.new_line();

    writer.unindent();

    // footer
    writer.add_line("#endif".to_string());

    // build product
    let mut path = context.output_folder.clone();
    path.push("hlsl/structs/");
    path.push(&struct_name);
    path.set_extension("hlsl");

    Product {
        path: path,
        content: writer.to_string(),
    }
}

fn get_descriptor_declaration(descriptor: &Descriptor) -> String {
    let typestring: String = match &descriptor.def {
        DescriptorDef::Sampler => "SamplerState ".to_owned(),
        DescriptorDef::ConstantBuffer(cb_def) => {
            format!(
                "ConstantBuffer<{}>",
                get_hlsl_typestring(&cb_def.inner_type)
            )
        }
        DescriptorDef::StructuredBuffer(sb_def) => {
            format!(
                "StructuredBuffer<{}>",
                get_hlsl_typestring(&sb_def.inner_type)
            )
        }
        DescriptorDef::RWStructuredBuffer(sb_def) => {
            format!(
                "RWStructuredBuffer<{}>",
                get_hlsl_typestring(&sb_def.inner_type)
            )
        }
        DescriptorDef::ByteAddressBuffer => "ByteAddressBuffer".to_owned(),
        DescriptorDef::RWByteAddressBuffer => "RWByteAddressBuffer".to_owned(),
        DescriptorDef::Texture2D(t_def) => {
            format!("Texture2D<{}>", get_hlsl_typestring(&t_def.inner_type))
        }
        DescriptorDef::RWTexture2D(t_def) => {
            format!("RWTexture2D<{}>", get_hlsl_typestring(&t_def.inner_type))
        }
    };

    format!("{} {};", typestring, descriptor.name)
}

fn generate_hlsl_pipelinelayout(context: &CGenContext, pl_name: &str) -> Product {
    let def = context.model.pipelinelayouts().get(pl_name).unwrap();
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!(
        "#ifndef PIPELINELAYOUT_{}",
        def.name.to_uppercase()
    ));
    writer.add_line(format!(
        "#define PIPELINELAYOUT_{}",
        def.name.to_uppercase()
    ));
    writer.new_line();

    writer.indent();

    // include all type dependencies
    let deps = context
        .model
        .get_pipelinelayout_type_dependencies(pl_name)
        .unwrap();

    if !deps.is_empty() {
        for dep in deps.iter() {
            writer.add_line(format!("#include \"../structs/{}.hlsl\"", dep.to_string()));
        }
        writer.new_line();
    }

    // write all descriptorsets
    if !def.descriptorsets.is_empty() {
        for ds_id in def.descriptorsets.iter() {
            let ds = context.model.descriptorsets().get(ds_id).unwrap();
            writer.add_line(format!(
                "// DescriptorSet '{}' : freq '{}'",
                ds.name, ds.frequency
            ));

            for (idx, d) in ds.descriptors.iter().enumerate() {
                writer.add_line(format!("[[vk::binding({}, {})]]", idx, ds.frequency));
                writer.add_line(get_descriptor_declaration(d));
            }
        }
        writer.new_line();
    }

    writer.unindent();

    // footer
    writer.add_line("#endif".to_string());

    //
    let mut path = context.output_folder.clone();
    path.push("hlsl/pipelinelayouts/");
    path.push(&pl_name);
    path.set_extension("hlsl");

    Product {
        path,
        content: writer.to_string(),
    }
}

struct Product {
    path: PathBuf,
    content: String,
}

impl Product {
    fn write_to_disk(&self) -> Result<()> {
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&self.path.parent().unwrap())?;

        let file_content = self.content.to_string();

        let mut output = std::fs::File::create(&self.path)?;
        output.write(&file_content.as_bytes())?;

        Ok(())
    }
}

fn run(file_path: &Path) -> Result<()> {
    let model = from_yaml(&file_path)?;

    let context = CGenContext {
        output_folder: PathBuf::from_str("d:/cgen_test/").unwrap(),
        model: &model,
    };

    //...

    let mut products = Vec::new();

    for item in model.structs().iter() {
        let product = generate_hlsl_struct(&context, &item.name);
        products.push(product);
    }

    for item in model.pipelinelayouts().iter() {
        let product = generate_hlsl_pipelinelayout(&context, &item.name);
        products.push(product);
    }

    // Write to disk

    for product in &products {
        product.write_to_disk()?;
    }

    Ok(())
}

#[test]
fn dump_model() {
    // // display model: structs
    // for s in mdl.structs().iter() {
    //     println!("struct {}", s.name);

    //     for m in &s.members {
    //         println!("- {} : {}", m.name, ToString::to_string(&m.cgen_type));
    //     }
    // }

    // // display model: descriptorsets
    // for ds in mdl.descriptorsets().iter() {
    //     println!("descriptorset {} : freq {}", ds.name, ds.frequency);

    //     for d in &ds.descriptors {
    //         print!("- {} : ", d.name);
    //         match &d.def {
    //             graphics_cgen::model::DescriptorDef::Sampler => {
    //                 println!("Sampler");
    //             }
    //             graphics_cgen::model::DescriptorDef::ConstantBuffer(def) => {
    //                 println!("ConstantBuffer : {}", ToString::to_string(&def.inner_type));
    //             }
    //             graphics_cgen::model::DescriptorDef::StructuredBuffer(def) => {
    //                 println!(
    //                     "StructuredBuffer : {}",
    //                     ToString::to_string(&def.inner_type)
    //                 );
    //             }
    //             graphics_cgen::model::DescriptorDef::RWStructuredBuffer(def) => {
    //                 println!(
    //                     "RWStructuredBuffer : {}",
    //                     ToString::to_string(&def.inner_type)
    //                 );
    //             }
    //             graphics_cgen::model::DescriptorDef::ByteAddressBuffer => {
    //                 println!("ByteAddressBuffer");
    //             }
    //             graphics_cgen::model::DescriptorDef::RWByteAddressBuffer => {
    //                 println!("RWByteAddressBuffer");
    //             }
    //             graphics_cgen::model::DescriptorDef::Texture2D(def) => {
    //                 println!("Texture2D : {}", def.format.as_ref());
    //             }
    //             graphics_cgen::model::DescriptorDef::RWTexture2D(def) => {
    //                 println!("RWTexture2D : {}", def.format.as_ref());
    //             }
    //         }
    //     }
    // }

    // // display model: pipelinelayouts
    // for pl in mdl.pipelinelayouts().iter() {
    //     println!("pipelinelayout {}", pl.name);

    //     for ds in &pl.descriptorsets {
    //         println!("- descriptorset {}", ds);
    //     }

    //     for pc in &pl.pushconstants {
    //         println!(
    //             "- pushconstant {} : {}",
    //             pc.name,
    //             ToString::to_string(&pc.cgen_type)
    //         );
    //     }
    // }
}
mod test {

    use std::{collections::HashMap, sync::Arc};

    #[derive(Debug)]
    enum Type {
        Float,
        Extern(Struct)
    }

    #[derive(Debug)]
    struct Member {
        name: String,
        typ: Type
    }

    impl Member {
        fn new(name: String, t: Type) -> Self {
            Member {
                name,
                typ: t
            }
        }
    }

    #[derive(Debug)]
    struct StructDef {
        name: String,
        members: Vec<Member>
    }

    impl StructDef {
        fn new(name: String) -> StructDef {
            StructDef { 
                name, 
                members: Vec::new() 
            }
        }

        fn add_member(&mut self, name: String, t: Type) {
            self.members.push(Member::new(name, t));
        }
    }

    #[derive(Debug)]
    struct Struct {
        inner : Arc<StructDef>
    }

    #[derive(Debug)]
    struct Container {
        objs: HashMap<String, Arc<StructDef>>
    }

    impl Container {
        fn add(&mut self, s: StructDef) -> Struct {
            let name = s.name.clone();
            let a = Arc::new(s);
            self.objs.insert(name, a.clone());
            Struct { 
                inner: a
            }
        }

        fn get_type(&self, name: &str) -> Type {
            let result = match name {
                "Float" => { Type::Float }
                _ => {                    
                    Type::Extern( Struct { inner : self.objs.get(name).unwrap().clone() } )
                }
            };
            result
        }
    }
    
    pub fn test1() {
        let mut c = Container {
            objs : HashMap::new()
        };

        let mut type_a = StructDef::new("A".to_owned());        
        type_a.add_member( "ma".to_owned(), c.get_type("Float") );
        c.add(type_a);

        let mut type_b = StructDef::new("B".to_owned());        
        type_b.add_member( "mb".to_owned(), c.get_type("A") );
        c.add(type_b);

        dbg!( c );



    }

}

