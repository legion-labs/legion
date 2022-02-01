pub mod egui_shader_family {

    use lgn_embedded_fs::embedded_watched_file;
    use lgn_graphics_api::ShaderStageFlags;
    use lgn_graphics_cgen_runtime::{
        CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance, CGenShaderOption,
    };

    embedded_watched_file!(UI_SHADER, "gpu/shaders/ui.hlsl");

    pub const ID: CGenShaderFamilyID = 1;

    pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {
        id: ID,
        name: "EGui",
        path: UI_SHADER.path(),
    };

    pub const SHADER_OPTION_TOTO: CGenShaderOption = CGenShaderOption {
        shader_family_id: ID,
        index: 0,
        name: "TOTO",
    };

    pub const TOTO: u64 = 1u64 << SHADER_OPTION_TOTO.index;

    pub static SHADER_OPTIONS: [&CGenShaderOption; 1] = [&SHADER_OPTION_TOTO];

    pub static SHADER_INSTANCES: [CGenShaderInstance; 1] = [CGenShaderInstance {
        shader_family_id: ID,
        shader_option_mask: TOTO,
        shader_stage_flags: ShaderStageFlags::from_bits_truncate(
            ShaderStageFlags::VERTEX_FLAG.bits() | ShaderStageFlags::FRAGMENT_FLAG.bits(),
        ),
    }];
}
