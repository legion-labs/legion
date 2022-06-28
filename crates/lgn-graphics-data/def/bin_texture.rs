pub enum TextureFormat {
    /// encode RGB or RGBA channels into BC1 (4 bits per pixel).
    BC1 = 0,
    /// encode RGB or RGBA channels into BC3 (8 bits per pixel).
    BC3,
    /// Encode R channel into BC4 (4 bits per pixel)
    BC4,
    /// encode RGB or RGBA channels into BC3 (8 bits per pixel).
    BC7,
}

struct Mips {
    /// Mip chain pixel data of the image in hardware encoded form
    pub texel_data: serde_bytes::ByteBuf,
}

#[resource]
#[legion(runtime_only)]
pub struct BinTexture {
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// Desired HW texture format
    pub format: TextureFormat,
    /// Color encoding
    pub srgb: bool,
    /// Mip chain pixel data of the image in hardware encoded form
    pub mips: Vec<Mips>,
}
