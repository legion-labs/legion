use std::hash::Hash;

#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

/// GL ES 2.0-specific shader package. Can be used to create a ShaderModuleDef, which in turn is
/// used to initialize a shader module GPU object
///
/// It is a struct rather than an enum because these are not mutually exclusive
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum ShaderPackageGles2 {
    /// Raw uncompiled OpenGL ES 2.0 source code. Will be compiled at runtime.
    Src(String),
}

/// GL ES 3.0-specific shader package. Can be used to create a ShaderModuleDef, which in turn is
/// used to initialize a shader module GPU object
///
/// It is a struct rather than an enum because these are not mutually exclusive
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum ShaderPackageGles3 {
    /// Raw uncompiled OpenGL ES 3.0 source code. Will be compiled at runtime.
    Src(String),
}

/// Metal-specific shader package. Can be used to create a ShaderModuleDef, which in turn is
/// used to initialize a shader module GPU object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum ShaderPackageMetal {
    /// Raw uncompiled source code. Will be compiled at runtime.
    Src(String),
    /// Pre-built binary "metallib" file loaded into memory
    #[cfg_attr(feature = "serde-support", serde(with = "serde_bytes"))]
    LibBytes(Vec<u8>),
}

/// Vulkan-specific shader package. Can be used to create a ShaderModuleDef, which in turn is
/// used to initialize a shader module GPU object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum ShaderPackageVulkan {
    /// Raw SPV bytes, no alignment or endianness requirements.
    #[cfg_attr(feature = "serde-support", serde(with = "serde_bytes"))]
    SpvBytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
#[doc(hidden)]
pub enum ShaderPackageEmpty {
    Empty,
}

/// Owns data necessary to create a shader module in (optionally) multiple APIs.
///
/// This struct can be serialized/deserialized and is intended to allow asset pipeline to store
/// a shader module to be created at runtime. The package can optionally include data for multiple
/// APIs allowing a single file to be used with whatever API is found at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct ShaderPackage {
    pub gles2: Option<ShaderPackageGles2>,
    pub gles3: Option<ShaderPackageGles3>,
    pub metal: Option<ShaderPackageMetal>,
    pub vk: Option<ShaderPackageVulkan>,
}

impl ShaderPackage {
    /// Create a shader module def for use with a GL Device. Returns none if the package does
    /// not contain data necessary for GL ES 2.0
    #[cfg(feature = "rafx-gles2")]
    pub fn gles2_module_def(&self) -> Option<ShaderModuleDefGles2> {
        if let Some(gl) = self.gles2.as_ref() {
            Some(match gl {
                ShaderPackageGles2::Src(src) => ShaderModuleDefGles2::GlSrc(src),
            })
        } else {
            None
        }
    }

    /// Create a shader module def for use with a GL Device. Returns none if the package does
    /// not contain data necessary for GL ES 2.0
    #[cfg(feature = "rafx-gles3")]
    pub fn gles3_module_def(&self) -> Option<ShaderModuleDefGles3> {
        if let Some(gl) = self.gles3.as_ref() {
            Some(match gl {
                ShaderPackageGles3::Src(src) => ShaderModuleDefGles3::GlSrc(src),
            })
        } else {
            None
        }
    }

    /// Create a shader module def for use with a metal Device. Returns none if the package does
    /// not contain data necessary for metal
    #[cfg(feature = "rafx-metal")]
    pub fn metal_module_def(&self) -> Option<ShaderModuleDefMetal> {
        if let Some(metal) = self.metal.as_ref() {
            Some(match metal {
                ShaderPackageMetal::Src(src) => ShaderModuleDefMetal::MetalSrc(src),
                ShaderPackageMetal::LibBytes(lib) => ShaderModuleDefMetal::MetalLibBytes(lib),
            })
        } else {
            None
        }
    }

    /// Create a shader module def for use with a vulkan Device. Returns none if the package
    /// does not contain data necessary for vulkan
    #[cfg(feature = "rafx-vulkan")]
    pub fn vulkan_module_def(&self) -> Option<ShaderModuleDefVulkan> {
        if let Some(vk) = self.vk.as_ref() {
            Some(match vk {
                ShaderPackageVulkan::SpvBytes(bytes) => ShaderModuleDefVulkan::VkSpvBytes(bytes),
            })
        } else {
            None
        }
    }

    /// Create a shader module def for use with a vulkan Device. Returns none if the package
    /// does not contain data necessary for vulkan
    #[cfg(any(
        feature = "rafx-empty",
        not(any(
            feature = "rafx-metal",
            feature = "rafx-vulkan",
            feature = "rafx-gles2",
            feature = "rafx-gles3"
        ))
    ))]
    #[doc(hidden)]
    pub fn empty_module_def(&self) -> Option<ShaderModuleDefEmpty> {
        Some(ShaderModuleDefEmpty::Empty(Default::default()))
    }

    pub fn module_def(&self) -> ShaderModuleDef {
        ShaderModuleDef {
            #[cfg(feature = "rafx-gles2")]
            gles2: self.gles2_module_def(),
            #[cfg(feature = "rafx-gles3")]
            gles3: self.gles3_module_def(),
            #[cfg(feature = "rafx-metal")]
            metal: self.metal_module_def(),
            #[cfg(feature = "rafx-vulkan")]
            vk: self.vulkan_module_def(),
            #[cfg(any(
                feature = "rafx-empty",
                not(any(
                    feature = "rafx-metal",
                    feature = "rafx-vulkan",
                    feature = "rafx-gles2",
                    feature = "rafx-gles3"
                ))
            ))]
            #[doc(hidden)]
            empty: self.empty_module_def(),
        }
    }
}

/// Used to create a ShaderModule
///
/// This enum may be populated manually or created from a ShaderPackage.
#[derive(Copy, Clone, Hash)]
#[cfg(feature = "rafx-gles2")]
pub enum ShaderModuleDefGles2<'a> {
    /// GL source code
    GlSrc(&'a str),
}

/// Used to create a ShaderModule
///
/// This enum may be populated manually or created from a ShaderPackage.
#[derive(Copy, Clone, Hash)]
#[cfg(feature = "rafx-gles3")]
pub enum ShaderModuleDefGles3<'a> {
    /// GL source code
    GlSrc(&'a str),
}

/// Used to create a ShaderModule
///
/// This enum may be populated manually or created from a ShaderPackage.
#[derive(Copy, Clone, Hash)]
#[cfg(feature = "rafx-metal")]
pub enum ShaderModuleDefMetal<'a> {
    /// Metal source code
    MetalSrc(&'a str),
    /// Pre-compiled library loaded as bytes
    MetalLibBytes(&'a [u8]),
}

/// Used to create a ShaderModule
///
/// This enum may be populated manually or created from a ShaderPackage.
#[derive(Copy, Clone, Hash)]
#[cfg(feature = "rafx-vulkan")]
pub enum ShaderModuleDefVulkan<'a> {
    /// Raw SPV bytes, no alignment or endianness requirements.
    VkSpvBytes(&'a [u8]),
    /// Prepared SPV that's aligned and correct endian. No validation.
    VkSpvPrepared(&'a [u32]),
}

#[cfg(any(
    feature = "rafx-empty",
    not(any(
        feature = "rafx-metal",
        feature = "rafx-vulkan",
        feature = "rafx-gles2",
        feature = "rafx-gles3"
    ))
))]
#[derive(Copy, Clone, Hash)]
#[doc(hidden)]
pub enum ShaderModuleDefEmpty<'a> {
    Empty(std::marker::PhantomData<&'a u32>),
}

/// Used to create a ShaderModule
///
/// This enum may be populated manually or created from a ShaderPackage.
#[derive(Copy, Clone, Hash, Default)]
pub struct ShaderModuleDef<'a> {
    #[cfg(feature = "rafx-gles2")]
    pub gles2: Option<ShaderModuleDefGles2<'a>>,
    #[cfg(feature = "rafx-gles3")]
    pub gles3: Option<ShaderModuleDefGles3<'a>>,
    #[cfg(feature = "rafx-metal")]
    pub metal: Option<ShaderModuleDefMetal<'a>>,
    #[cfg(feature = "rafx-vulkan")]
    pub vk: Option<ShaderModuleDefVulkan<'a>>,
    #[cfg(any(
        feature = "rafx-empty",
        not(any(
            feature = "rafx-metal",
            feature = "rafx-vulkan",
            feature = "rafx-gles2",
            feature = "rafx-gles3"
        ))
    ))]
    #[doc(hidden)]
    pub empty: Option<ShaderModuleDefEmpty<'a>>,
}
