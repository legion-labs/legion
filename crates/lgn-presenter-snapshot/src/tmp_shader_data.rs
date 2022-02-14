// #![allow(dead_code)]

// use lgn_graphics_cgen_runtime::CGenRegistry;

// pub mod this_cgen {
//     use lgn_graphics_cgen_runtime::CGenCrateID;

//     pub const ID: CGenCrateID = CGenCrateID(2);
// }

// pub mod display_mapper_shader_family {

//     use lgn_embedded_fs::embedded_watched_file;
//     use lgn_graphics_api::ShaderStageFlags;
//     use lgn_graphics_cgen_runtime::{
//         CGenShaderDef, CGenShaderID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,
//     };

//     use super::this_cgen;

//     embedded_watched_file!(SHADER_PATH, "shaders/display_mapper.hlsl");

//     pub const ID: CGenShaderID = CGenShaderID::make(this_cgen::ID, 1);

//     pub static SHADER_FAMILY: CGenShaderDef = CGenShaderDef {
//         id: ID,
//         name: "DisplayMapper",
//         path: SHADER_PATH.path(),
//         options: &SHADER_OPTIONS,
//         instances: &SHADER_INSTANCES,
//     };

//     pub const NONE: u64 = 0;

//     pub static SHADER_OPTIONS: [CGenShaderOption; 0] = [];

//     pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
//         key: CGenShaderKey::make(ID, NONE),
//         stage_flags: ShaderStageFlags::from_bits_truncate(
//             ShaderStageFlags::VERTEX_FLAG.bits() | ShaderStageFlags::FRAGMENT_FLAG.bits(),
//         ),
//     }];
// }

// macro_rules! register_family {
//     ($registry:ident,  $family:ident) => {
//         $registry.shader_families.push(&$family::SHADER_FAMILY);
//     };
// }

// pub fn patch_cgen_registry(cgen_registry: &mut CGenRegistry) {
//     register_family!(cgen_registry, display_mapper_shader_family);
// }
