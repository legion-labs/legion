
#[derive(Debug, Clone)]
pub struct OutputId (u64);

#[derive(Debug, Clone)]
pub struct OutputDescriptor {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone)]
pub struct CreateOutput {
    pub id: OutputId,
    pub descriptor: OutputDescriptor,
}

#[derive(Debug, Clone)]
pub struct RenderOutputDestroyed;