#![allow(dead_code)]

use lgn_graphics_cgen_runtime::CGenRegistry;

pub mod this_cgen {
    use lgn_graphics_cgen_runtime::CGenCrateID;

    pub const ID: CGenCrateID = CGenCrateID(3);
}

pub mod rgb2yuv_shader_family {

    use lgn_embedded_fs::embedded_watched_file;
    use lgn_graphics_api::ShaderStageFlags;
    use lgn_graphics_cgen_runtime::{
        CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,
    };

    use super::this_cgen;

    embedded_watched_file!(SHADER_PATH, "shaders/rgb2yuv.hlsl");

    pub const ID: CGenShaderFamilyID = CGenShaderFamilyID::make(this_cgen::ID, 1);

    pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {
        id: ID,
        name: "Rgb2Yuv",
        path: SHADER_PATH.path(),
        options: &SHADER_OPTIONS,
        instances: &SHADER_INSTANCES,
    };

    pub const NONE: u64 = 0;

    pub static SHADER_OPTIONS: [CGenShaderOption; 0] = [];

    pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
        key: CGenShaderKey::make(ID, NONE),
        stage_flags: ShaderStageFlags::from_bits_truncate(ShaderStageFlags::COMPUTE_FLAG.bits()),
    }];
}

macro_rules! register_family {
    ($registry:ident,  $family:ident) => {
        $registry.shader_families.push(&$family::SHADER_FAMILY);
    };
}

pub fn patch_cgen_registry(cgen_registry: &mut CGenRegistry) {
    register_family!(cgen_registry, rgb2yuv_shader_family);
}
