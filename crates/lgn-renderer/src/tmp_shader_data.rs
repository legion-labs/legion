#![allow(dead_code)]

use lgn_graphics_cgen_runtime::CGenRegistry;

pub mod this_cgen {
    use lgn_graphics_cgen_runtime::CGenCrateID;

    pub const ID: CGenCrateID = CGenCrateID(1);
}

pub mod egui_shader_family {

    use lgn_embedded_fs::embedded_watched_file;
    use lgn_graphics_api::ShaderStageFlags;
    use lgn_graphics_cgen_runtime::{
        CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,
    };

    use super::this_cgen;

    embedded_watched_file!(SHADER_PATH, "gpu/shaders/ui.hlsl");

    pub const ID: CGenShaderFamilyID = CGenShaderFamilyID::make(this_cgen::ID, 1);

    pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {
        id: ID,
        name: "EGui",
        path: SHADER_PATH.path(),
        options: &SHADER_OPTIONS,
    };

    pub const NONE: u64 = 0;
    pub const TOTO: u64 = 1u64 << 0;

    pub static SHADER_OPTIONS: [CGenShaderOption; 1] = [CGenShaderOption {        
        index: 0,
        name: "TOTO",
    }];

    pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
        key: CGenShaderKey::make(ID, TOTO),
        stage_flags: ShaderStageFlags::from_bits_truncate(
            ShaderStageFlags::VERTEX_FLAG.bits() | ShaderStageFlags::FRAGMENT_FLAG.bits(),
        ),
    }];
}

pub mod shader_shader_family {

    use lgn_embedded_fs::embedded_watched_file;
    use lgn_graphics_api::ShaderStageFlags;
    use lgn_graphics_cgen_runtime::{
        CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,
    };

    use super::this_cgen;

    embedded_watched_file!(SHADER_PATH, "gpu/shaders/shader.hlsl");

    pub const ID: CGenShaderFamilyID = CGenShaderFamilyID::make(this_cgen::ID, 2);

    pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {
        id: ID,
        name: "Shader",
        path: SHADER_PATH.path(),
        options: &SHADER_OPTIONS,
    };

    pub const NONE: u64 = 0;

    pub static SHADER_OPTIONS: [CGenShaderOption; 0] = [];

    pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
        key: CGenShaderKey::make(ID, NONE),
        stage_flags: ShaderStageFlags::from_bits_truncate(
            ShaderStageFlags::VERTEX_FLAG.bits() | ShaderStageFlags::FRAGMENT_FLAG.bits(),
        ),
    }];
}

pub mod picking_shader_family {

    use lgn_embedded_fs::embedded_watched_file;
    use lgn_graphics_api::ShaderStageFlags;
    use lgn_graphics_cgen_runtime::{
        CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,
    };

    use super::this_cgen;

    embedded_watched_file!(SHADER_PATH, "gpu/shaders/picking.hlsl");

    pub const ID: CGenShaderFamilyID = CGenShaderFamilyID::make(this_cgen::ID, 3);

    pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {
        id: ID,
        name: "Picking",
        path: SHADER_PATH.path(),
        options: &SHADER_OPTIONS,
    };

    pub const NONE: u64 = 0;

    pub static SHADER_OPTIONS: [CGenShaderOption; 0] = [];

    pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
        key: CGenShaderKey::make(ID, NONE),
        stage_flags: ShaderStageFlags::from_bits_truncate(
            ShaderStageFlags::VERTEX_FLAG.bits() | ShaderStageFlags::FRAGMENT_FLAG.bits(),
        ),
    }];
}

pub mod const_color_shader_family {

    use lgn_embedded_fs::embedded_watched_file;
    use lgn_graphics_api::ShaderStageFlags;
    use lgn_graphics_cgen_runtime::{
        CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,
    };

    use super::this_cgen;

    embedded_watched_file!(SHADER_PATH, "gpu/shaders/const_color.hlsl");

    pub const ID: CGenShaderFamilyID = CGenShaderFamilyID::make(this_cgen::ID, 4);

    pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {
        id: ID,
        name: "ConstColor",
        path: SHADER_PATH.path(),
        options: &SHADER_OPTIONS,
    };

    pub const NONE: u64 = 0;

    pub static SHADER_OPTIONS: [CGenShaderOption; 0] = [];

    pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
        key: CGenShaderKey::make(ID, NONE),
        stage_flags: ShaderStageFlags::from_bits_truncate(
            ShaderStageFlags::VERTEX_FLAG.bits() | ShaderStageFlags::FRAGMENT_FLAG.bits(),
        ),
    }];
}

macro_rules! register_family {
    ($registry:ident,  $family:ident) => {
        $registry.shader_families.push(&$family::SHADER_FAMILY);

        $family::SHADER_INSTANCES
            .iter()
            .for_each(|x| $registry.shader_instances.push(x));
    };
}

pub fn patch_cgen_registry(cgen_registry: &mut CGenRegistry) {
    register_family!(cgen_registry, egui_shader_family);
    register_family!(cgen_registry, shader_shader_family);
    register_family!(cgen_registry, picking_shader_family);
    register_family!(cgen_registry, const_color_shader_family);
}
