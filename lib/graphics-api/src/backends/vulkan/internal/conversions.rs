use crate::{AddressMode, BlendFactor, BlendOp, ColorClearValue, ColorFlags, CompareOp, CullMode, DepthStencilClearValue, FillMode, FilterType, FrontFace, IndexType, LoadOp, MemoryUsage, MipMapMode, PrimitiveTopology, SampleCount, ShaderStageFlags, StencilOp, StoreOp, TextureTiling, VertexAttributeRate, ViewType};
use ash::vk;

impl From<SampleCount> for vk::SampleCountFlags {
    fn from(val: SampleCount) -> Self {
        match val {
            SampleCount::SampleCount1 => Self::TYPE_1,
            SampleCount::SampleCount2 => Self::TYPE_2,
            SampleCount::SampleCount4 => Self::TYPE_4,
            SampleCount::SampleCount8 => Self::TYPE_8,
            SampleCount::SampleCount16 => Self::TYPE_16,
        }
    }
}

impl From<TextureTiling> for vk::ImageTiling {
    fn from(val: TextureTiling) -> Self {
        match val {
            TextureTiling::Optimal => Self::OPTIMAL,
            TextureTiling::Linear => Self::LINEAR,
        }
    }
}

impl From<ColorFlags> for vk::ColorComponentFlags {
    fn from(val: ColorFlags) -> Self {
        let mut flags = Self::empty();
        if val.intersects(ColorFlags::RED) {
            flags |= Self::R;
        }
        if val.intersects(ColorFlags::GREEN) {
            flags |= Self::G;
        }
        if val.intersects(ColorFlags::BLUE) {
            flags |= Self::B;
        }
        if val.intersects(ColorFlags::ALPHA) {
            flags |= Self::A;
        }
        flags
    }
}

impl From<MemoryUsage> for vk_mem::MemoryUsage {
    fn from(val: MemoryUsage) -> Self {
        match val {
            MemoryUsage::Unknown => Self::Unknown,
            MemoryUsage::GpuOnly => Self::GpuOnly,
            MemoryUsage::CpuOnly => Self::CpuOnly,
            MemoryUsage::CpuToGpu => Self::CpuToGpu,
            MemoryUsage::GpuToCpu => Self::GpuToCpu,
        }
    }
}

impl From<ShaderStageFlags> for vk::ShaderStageFlags {
    fn from(val: ShaderStageFlags) -> Self {
        let mut result = Self::empty();

        if val.intersects(ShaderStageFlags::VERTEX) {
            result |= Self::VERTEX;
        }

        if val.intersects(ShaderStageFlags::TESSELLATION_CONTROL) {
            result |= Self::TESSELLATION_CONTROL;
        }

        if val.intersects(ShaderStageFlags::TESSELLATION_EVALUATION) {
            result |= Self::TESSELLATION_EVALUATION;
        }

        if val.intersects(ShaderStageFlags::GEOMETRY) {
            result |= Self::GEOMETRY;
        }

        if val.intersects(ShaderStageFlags::FRAGMENT) {
            result |= Self::FRAGMENT;
        }

        if val.intersects(ShaderStageFlags::COMPUTE) {
            result |= Self::COMPUTE;
        }

        if val.contains(ShaderStageFlags::ALL_GRAPHICS) {
            result |= Self::ALL_GRAPHICS;
        }

        result
    }
}

impl From<VertexAttributeRate> for vk::VertexInputRate {
    fn from(val: VertexAttributeRate) -> Self {
        match val {
            VertexAttributeRate::Vertex => Self::VERTEX,
            VertexAttributeRate::Instance => Self::INSTANCE,
        }
    }
}

impl From<LoadOp> for vk::AttachmentLoadOp {
    fn from(val: LoadOp) -> Self {
        match val {
            LoadOp::DontCare => Self::DONT_CARE,
            LoadOp::Load => Self::LOAD,
            LoadOp::Clear => Self::CLEAR,
        }
    }
}

impl From<StoreOp> for vk::AttachmentStoreOp {
    fn from(val: StoreOp) -> Self {
        match val {
            StoreOp::DontCare => Self::DONT_CARE,
            StoreOp::Store => Self::STORE,
        }
    }
}

impl From<PrimitiveTopology> for vk::PrimitiveTopology {
    fn from(val: PrimitiveTopology) -> Self {
        match val {
            PrimitiveTopology::PointList => Self::POINT_LIST,
            PrimitiveTopology::LineList => Self::LINE_LIST,
            PrimitiveTopology::LineStrip => Self::LINE_STRIP,
            PrimitiveTopology::TriangleList => Self::TRIANGLE_LIST,
            PrimitiveTopology::TriangleStrip => Self::TRIANGLE_STRIP,
            PrimitiveTopology::PatchList => Self::PATCH_LIST,
        }
    }
}

impl From<IndexType> for vk::IndexType {
    fn from(val: IndexType) -> Self {
        match val {
            IndexType::Uint32 => Self::UINT32,
            IndexType::Uint16 => Self::UINT16,
        }
    }
}

impl From<BlendFactor> for vk::BlendFactor {
    fn from(val: BlendFactor) -> Self {
        match val {
            BlendFactor::Zero => Self::ZERO,
            BlendFactor::One => Self::ONE,
            BlendFactor::SrcColor => Self::SRC_COLOR,
            BlendFactor::OneMinusSrcColor => Self::ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => Self::DST_COLOR,
            BlendFactor::OneMinusDstColor => Self::ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => Self::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => Self::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => Self::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => Self::ONE_MINUS_DST_ALPHA,
            BlendFactor::SrcAlphaSaturate => Self::SRC_ALPHA_SATURATE,
            BlendFactor::ConstantColor => Self::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => Self::ONE_MINUS_CONSTANT_COLOR,
        }
    }
}

impl From<BlendOp> for vk::BlendOp {
    fn from(val: BlendOp) -> Self {
        match val {
            BlendOp::Add => Self::ADD,
            BlendOp::Subtract => Self::SUBTRACT,
            BlendOp::ReverseSubtract => Self::REVERSE_SUBTRACT,
            BlendOp::Min => Self::MIN,
            BlendOp::Max => Self::MAX,
        }
    }
}

impl From<CompareOp> for vk::CompareOp {
    fn from(val: CompareOp) -> Self {
        match val {
            CompareOp::Never => Self::NEVER,
            CompareOp::Less => Self::LESS,
            CompareOp::Equal => Self::EQUAL,
            CompareOp::LessOrEqual => Self::LESS_OR_EQUAL,
            CompareOp::Greater => Self::GREATER,
            CompareOp::NotEqual => Self::NOT_EQUAL,
            CompareOp::GreaterOrEqual => Self::GREATER_OR_EQUAL,
            CompareOp::Always => Self::ALWAYS,
        }
    }
}

impl From<StencilOp> for vk::StencilOp {
    fn from(val: StencilOp) -> Self {
        match val {
            StencilOp::Keep => Self::KEEP,
            StencilOp::Zero => Self::ZERO,
            StencilOp::Replace => Self::REPLACE,
            StencilOp::IncrementAndClamp => Self::INCREMENT_AND_CLAMP,
            StencilOp::DecrementAndClamp => Self::DECREMENT_AND_CLAMP,
            StencilOp::Invert => Self::INVERT,
            StencilOp::IncrementAndWrap => Self::INCREMENT_AND_WRAP,
            StencilOp::DecrementAndWrap => Self::DECREMENT_AND_WRAP,
        }
    }
}

impl From<CullMode> for vk::CullModeFlags {
    fn from(val: CullMode) -> Self {
        match val {
            CullMode::None => Self::NONE,
            CullMode::Back => Self::BACK,
            CullMode::Front => Self::FRONT,
        }
    }
}

impl From<FrontFace> for vk::FrontFace {
    fn from(val: FrontFace) -> Self {
        match val {
            FrontFace::CounterClockwise => Self::COUNTER_CLOCKWISE,
            FrontFace::Clockwise => Self::CLOCKWISE,
        }
    }
}

impl From<FillMode> for vk::PolygonMode {
    fn from(val: FillMode) -> Self {
        match val {
            FillMode::Solid => Self::FILL,
            FillMode::Wireframe => Self::LINE,
        }
    }
}

impl From<FilterType> for vk::Filter {
    fn from(val: FilterType) -> Self {
        match val {
            FilterType::Nearest => Self::NEAREST,
            FilterType::Linear => Self::LINEAR,
        }
    }
}

impl From<AddressMode> for vk::SamplerAddressMode {
    fn from(val: AddressMode) -> Self {
        match val {
            AddressMode::Mirror => Self::MIRRORED_REPEAT,
            AddressMode::Repeat => Self::REPEAT,
            AddressMode::ClampToEdge => Self::CLAMP_TO_EDGE,
            AddressMode::ClampToBorder => Self::CLAMP_TO_BORDER,
        }
    }
}

impl From<MipMapMode> for vk::SamplerMipmapMode {
    fn from(val: MipMapMode) -> Self {
        match val {
            MipMapMode::Nearest => Self::NEAREST,
            MipMapMode::Linear => Self::LINEAR,
        }
    }
}

impl From<ColorClearValue> for vk::ClearValue {
    fn from(val: ColorClearValue) -> Self {
        Self {
            color: vk::ClearColorValue { float32: val.0 },
        }
    }
}

impl From<DepthStencilClearValue> for vk::ClearValue {
    fn from(val: DepthStencilClearValue) -> Self {
        Self {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: val.depth,
                stencil: val.stencil,
            },
        }
    }
}

impl From<ViewType> for vk::ImageViewType {
    fn from(val: ViewType) -> Self {
        match val {
            ViewType::ViewType2d => vk::ImageViewType::TYPE_2D,
            ViewType::ViewType2darray => vk::ImageViewType::TYPE_2D_ARRAY,
            ViewType::ViewTypeCube => vk::ImageViewType::CUBE,
            ViewType::ViewTypeCubeArray => vk::ImageViewType::CUBE_ARRAY,
            ViewType::ViewType3d => vk::ImageViewType::TYPE_3D,
        }
    }
}
