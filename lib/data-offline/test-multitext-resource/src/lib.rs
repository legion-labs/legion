use legion_data_offline::{
    asset::AssetPathId,
    resource::{Resource, ResourceProcessor, ResourceType},
};

pub const TYPE_ID: ResourceType = ResourceType::new(b"multitext_resource");

pub struct MultiTextResource {
    pub text_list: Vec<String>,
}

impl Resource for MultiTextResource {}

pub struct MultiTextResourceProc {}

impl ResourceProcessor for MultiTextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(MultiTextResource { text_list: vec![] })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<AssetPathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<MultiTextResource>().unwrap();
        let mut size = writer.write(&resource.text_list.len().to_ne_bytes())?;
        for content in &resource.text_list {
            size += writer.write(&content.len().to_ne_bytes())?;
            size += writer.write(content.as_bytes())?;
        }

        Ok(size)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let mut boxed = self.new_resource();
        let resource = boxed.downcast_mut::<MultiTextResource>().unwrap();

        let mut buf = 0usize.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        let count = usize::from_ne_bytes(buf);
        for _ in 0..count {
            reader.read_exact(&mut buf)?;
            let text_len = usize::from_ne_bytes(buf);
            let mut buf = Box::new(vec![0u8; text_len]);
            reader.read_exact(&mut buf)?;
            resource
                .text_list
                .push(String::from_utf8(buf.to_vec()).unwrap());
        }
        Ok(boxed)
    }
}
