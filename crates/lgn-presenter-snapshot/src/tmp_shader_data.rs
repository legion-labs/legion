#![allow(dead_code)]

use lgn_graphics_cgen_runtime::CGenRegistry;

pub mod this_cgen {
    pub const ID: u64 = 2;
}

pub mod display_mapper_shader_family {

    use lgn_embedded_fs::embedded_watched_file;
    use lgn_graphics_api::ShaderStageFlags;
    use lgn_graphics_cgen_runtime::{
        shader_family_id, CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance,
        CGenShaderOption,
    };

    use super::this_cgen;

    embedded_watched_file!(SHADER_PATH, "shaders/display_mapper.hlsl");

    pub const ID: CGenShaderFamilyID = shader_family_id(this_cgen::ID, 1);

    pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {
        id: ID,
        name: "DisplayMapper",
        path: SHADER_PATH.path(),
    };

    pub const NONE: u64 = 0;

    pub static SHADER_OPTIONS: [&CGenShaderOption; 0] = [];

    pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
        shader_family_id: ID,
        shader_option_mask: NONE,
        shader_stage_flags: ShaderStageFlags::from_bits_truncate(
            ShaderStageFlags::VERTEX_FLAG.bits() | ShaderStageFlags::FRAGMENT_FLAG.bits(),
        ),
    }];
}

macro_rules! register_family {
    ($registry:ident,  $family:ident) => {
        $registry.shader_families.push(&$family::SHADER_FAMILY);

        $family::SHADER_OPTIONS
            .iter()
            .for_each(|x| $registry.shader_options.push(x));

        $family::SHADER_INSTANCES
            .iter()
            .for_each(|x| $registry.shader_instances.push(x));
    };
}

pub fn patch_cgen_registry(cgen_registry: &mut CGenRegistry) {
    register_family!(cgen_registry, display_mapper_shader_family);
}
