use resources::{Resource, ResourcePathId, ResourceProcessor, ResourceType};

pub const TYPE_ID: ResourceType = ResourceType::new(b"mock_resource");

pub struct MockResource {
    pub magic_value: i32,
}

impl Resource for MockResource {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct MockResourceProc {}

impl ResourceProcessor for MockResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(MockResource { magic_value: 0 })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.as_any().downcast_ref::<MockResource>().unwrap();
        let size = writer.write(&resource.magic_value.to_ne_bytes())?;
        assert_eq!(size, std::mem::size_of::<i32>());
        Ok(size)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let mut boxed = self.new_resource();
        let mut resource = boxed.as_any_mut().downcast_mut::<MockResource>().unwrap();

        let mut buf = 0i32.to_ne_bytes();
        reader.read_exact(&mut buf)?;
        resource.magic_value = i32::from_ne_bytes(buf);
        Ok(boxed)
    }
}
