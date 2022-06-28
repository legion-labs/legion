pub enum TextureType {
    /// 2d texture.
    _2D,
}

#[resource]
#[legion(runtime_only)]
pub struct RawTexture {
    pub kind: TextureType,
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// Texture pixel data.
    pub rgba: serde_bytes::ByteBuf,
}
