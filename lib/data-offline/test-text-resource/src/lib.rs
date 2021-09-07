use legion_data_offline::{
    asset::AssetPathId,
    resource::{Resource, ResourceProcessor, ResourceType},
};

pub const TYPE_ID: ResourceType = ResourceType::new(b"text_resource");

#[derive(Resource)]
pub struct TextResource {
    pub content: String,
}

pub struct TextResourceProc {}

impl ResourceProcessor for TextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(TextResource {
            content: String::from("7"),
        })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<AssetPathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<TextResource>().unwrap();
        let size = writer.write(&resource.content.len().to_ne_bytes())?;
        assert_eq!(size, std::mem::size_of::<usize>());
        let written = writer.write(resource.content.as_bytes())?;
        assert_eq!(written, resource.content.len());
        Ok(size + written)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let mut boxed = self.new_resource();
        let mut resource = boxed.downcast_mut::<TextResource>().unwrap();

        let mut buf = 0usize.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        let len = usize::from_ne_bytes(buf);
        let mut buf = Box::new(vec![0u8; len]);
        reader.read_exact(&mut buf)?;
        resource.content = String::from_utf8(buf.to_vec()).unwrap();
        Ok(boxed)
    }
}
