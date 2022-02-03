use lgn_utils::decimal::DecimalF32;
use std::hash::{Hash, Hasher};

use crate::backends::BackendPipeline;
use crate::{deferred_drop::Drc, GfxResult, PipelineType, RootSignature};
use crate::{
    BlendFactor, BlendOp, BlendStateTargets, ColorFlags, CompareOp, CullMode, DeviceContext,
    FillMode, Format, FrontFace, PrimitiveTopology, SampleCount, Shader, StencilOp,
    VertexAttributeRate,
};

pub const MAX_VERTEX_ATTRIBUTES: usize = 8;

/// Describes an attribute within a `VertexLayout`
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexLayoutAttribute {
    /// Format of the attribute
    pub format: Format,
    /// Which buffer the attribute is contained in
    pub buffer_index: u32,
    /// Affects what input variable within the shader the attribute is assigned
    pub location: u32,
    /// The byte offset of the attribute within the buffer
    pub byte_offset: u32,
}

/// Describes a buffer that provides vertex attribute data (See `VertexLayout`)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexLayoutBuffer {
    pub stride: u32,
    pub rate: VertexAttributeRate,
}

/// Describes how vertex attributes are laid out within one or more buffers
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexLayout {
    pub attributes: [Option<VertexLayoutAttribute>; MAX_VERTEX_ATTRIBUTES],
    pub buffers: [Option<VertexLayoutBuffer>; MAX_VERTEX_ATTRIBUTES],
}

/// Affects depth testing and stencil usage. Commonly used to enable
/// "Z-buffering".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthState {
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_op: CompareOp,
    pub stencil_test_enable: bool,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,
    pub front_depth_fail_op: StencilOp,
    pub front_stencil_compare_op: CompareOp,
    pub front_stencil_fail_op: StencilOp,
    pub front_stencil_pass_op: StencilOp,
    pub back_depth_fail_op: StencilOp,
    pub back_stencil_compare_op: CompareOp,
    pub back_stencil_fail_op: StencilOp,
    pub back_stencil_pass_op: StencilOp,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            depth_test_enable: false,
            depth_write_enable: false,
            depth_compare_op: CompareOp::LessOrEqual,
            stencil_test_enable: false,
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
            front_depth_fail_op: StencilOp::default(),
            front_stencil_compare_op: CompareOp::Always,
            front_stencil_fail_op: StencilOp::default(),
            front_stencil_pass_op: StencilOp::default(),
            back_depth_fail_op: StencilOp::default(),
            back_stencil_compare_op: CompareOp::Always,
            back_stencil_fail_op: StencilOp::default(),
            back_stencil_pass_op: StencilOp::default(),
        }
    }
}

/// Affects rasterization, commonly used to enable backface culling or wireframe
/// rendering
#[derive(Debug, Clone, Copy)]
pub struct RasterizerState {
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub fill_mode: FillMode,
    pub depth_bias: f32,
    pub depth_bias_slope_scaled: f32,
    pub depth_clamp_enable: bool,
    pub multisample: bool,
    pub scissor: bool,
    pub line_width: f32,
    // Hash implemented manually below, don't forget to update it!
}

impl Eq for RasterizerState {}

impl PartialEq for RasterizerState {
    fn eq(&self, other: &Self) -> bool {
        self.cull_mode == other.cull_mode
            && self.front_face == other.front_face
            && self.fill_mode == other.fill_mode
            && DecimalF32(self.depth_bias) == DecimalF32(other.depth_bias)
            && DecimalF32(self.depth_bias_slope_scaled) == DecimalF32(other.depth_bias_slope_scaled)
            && self.depth_clamp_enable == other.depth_clamp_enable
            && self.multisample == other.multisample
            && self.scissor == other.scissor
            && self.line_width == other.line_width
    }
}

impl Hash for RasterizerState {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        self.cull_mode.hash(&mut state);
        self.front_face.hash(&mut state);
        self.fill_mode.hash(&mut state);
        DecimalF32(self.depth_bias).hash(&mut state);
        DecimalF32(self.depth_bias_slope_scaled).hash(&mut state);
        self.depth_clamp_enable.hash(&mut state);
        self.multisample.hash(&mut state);
        self.scissor.hash(&mut state);
        DecimalF32(self.line_width).hash(&mut state);
    }
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::default(),
            front_face: FrontFace::default(),
            fill_mode: FillMode::default(),
            depth_bias: 0.0,
            depth_bias_slope_scaled: 0.0,
            depth_clamp_enable: false,
            multisample: false,
            scissor: false,
            line_width: 1.0,
        }
    }
}

/// Configures blend state for a particular render target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlendStateRenderTarget {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub src_factor_alpha: BlendFactor,
    pub dst_factor_alpha: BlendFactor,
    pub blend_op: BlendOp,
    pub blend_op_alpha: BlendOp,
    pub masks: ColorFlags,
}

impl Default for BlendStateRenderTarget {
    fn default() -> Self {
        Self {
            blend_op: BlendOp::Add,
            blend_op_alpha: BlendOp::Add,
            src_factor: BlendFactor::One,
            src_factor_alpha: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            dst_factor_alpha: BlendFactor::Zero,
            masks: ColorFlags::ALL,
        }
    }
}

impl BlendStateRenderTarget {
    pub fn default_alpha_disabled() -> Self {
        Self::default()
    }

    pub fn default_alpha_enabled() -> Self {
        Self {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            src_factor_alpha: BlendFactor::One,
            dst_factor_alpha: BlendFactor::Zero,
            blend_op: BlendOp::Add,
            blend_op_alpha: BlendOp::Add,
            masks: ColorFlags::ALL,
        }
    }
}

impl BlendStateRenderTarget {
    pub fn blend_enabled(self) -> bool {
        self.src_factor != BlendFactor::One
            || self.src_factor_alpha != BlendFactor::One
            || self.dst_factor != BlendFactor::Zero
            || self.dst_factor_alpha != BlendFactor::Zero
    }
}

/// Affects the way the result of a pixel shader is blended with a value it will
/// overwrite. Commonly used to enable "alpha-blending".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlendState {
    /// Individual blend states for blend targets
    pub render_target_blend_states: Vec<BlendStateRenderTarget>,

    /// Indicates which blend targets to affect. Blend targets with unset bits
    /// are left in default state.
    pub render_target_mask: BlendStateTargets,

    /// If false, `render_target_blend_states[0]` will apply to all render
    /// targets indicated by `render_target_mask`. If true, we index into
    /// `render_target_blend_states` based on the render target's index.
    pub independent_blend: bool,
}

impl BlendState {
    pub fn default_alpha_disabled() -> Self {
        Self {
            render_target_blend_states: vec![BlendStateRenderTarget::default_alpha_disabled()],
            render_target_mask: BlendStateTargets::BLEND_STATE_TARGET_ALL,
            independent_blend: false,
        }
    }

    pub fn default_alpha_enabled() -> Self {
        Self {
            render_target_blend_states: vec![BlendStateRenderTarget::default_alpha_enabled()],
            render_target_mask: BlendStateTargets::BLEND_STATE_TARGET_ALL,
            independent_blend: false,
        }
    }
}

impl Default for BlendState {
    fn default() -> Self {
        Self::default_alpha_disabled()
    }
}

impl BlendState {
    pub fn verify(&self, color_attachment_count: usize) {
        if self.independent_blend {
            assert_eq!(self.render_target_blend_states.len(), color_attachment_count,
                "If BlendState::independent_blend is true, BlendState::render_target_blend_states length must match color attachment count");
        } else {
            assert_eq!(self.render_target_blend_states.len(), 1,
                "If BlendState::independent_blend is false, BlendState::render_target_blend_states must be 1");
        }
    }
}

/// Used to create a `Pipeline` for graphics operations
pub struct GraphicsPipelineDef<'a> {
    pub shader: &'a Shader,
    pub root_signature: &'a RootSignature,
    pub vertex_layout: &'a VertexLayout,
    pub blend_state: &'a BlendState,
    pub depth_state: &'a DepthState,
    pub rasterizer_state: &'a RasterizerState,
    pub primitive_topology: PrimitiveTopology,
    pub color_formats: &'a [Format],
    pub depth_stencil_format: Option<Format>,
    pub sample_count: SampleCount,
}

/// Used to create a `Pipeline` for compute operations
pub struct ComputePipelineDef<'a> {
    pub shader: &'a Shader,
    pub root_signature: &'a RootSignature,
}

pub(crate) struct PipelineInner {
    root_signature: RootSignature,
    pipeline_type: PipelineType,
    pub(crate) backend_pipeline: BackendPipeline,
}

impl Drop for PipelineInner {
    fn drop(&mut self) {
        self.backend_pipeline
            .destroy(self.root_signature.device_context());
    }
}

#[derive(Clone)]
pub struct Pipeline {
    pub(crate) inner: Drc<PipelineInner>,
}

impl Pipeline {
    pub fn new_graphics_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &GraphicsPipelineDef<'_>,
    ) -> GfxResult<Self> {
        let backend_pipeline =
            BackendPipeline::new_graphics_pipeline(device_context, pipeline_def)?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(PipelineInner {
                pipeline_type: PipelineType::Graphics,
                root_signature: pipeline_def.root_signature.clone(),
                backend_pipeline,
            }),
        })
    }

    pub fn new_compute_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &ComputePipelineDef<'_>,
    ) -> GfxResult<Self> {
        let backend_pipeline = BackendPipeline::new_compute_pipeline(device_context, pipeline_def)?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(PipelineInner {
                pipeline_type: PipelineType::Compute,
                root_signature: pipeline_def.root_signature.clone(),
                backend_pipeline,
            }),
        })
    }

    pub fn pipeline_type(&self) -> PipelineType {
        self.inner.pipeline_type
    }

    pub fn root_signature(&self) -> &RootSignature {
        &self.inner.root_signature
    }
}
