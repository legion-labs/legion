//! This module defines a test resource.
//!
//! It is used to test the data compilation process until we have a proper resource available.

use std::str::FromStr;

use legion_data_offline::{
    asset::AssetPathId,
    resource::{Resource, ResourceProcessor, ResourceType},
};

/// Type id of test resource. Used until we have proper resource types.
pub const TYPE_ID: ResourceType = ResourceType::new(b"test_resource");

/// Resource temporarily used for testing.
///
/// To be removed once real resource types exist.
pub struct TestResource {
    /// Resource's content.
    pub content: String,
    /// Resource's build dependencies.
    pub build_deps: Vec<AssetPathId>,
}
impl Resource for TestResource {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// [`TestResource`]'s resource processor temporarily used for testings.
///
/// To be removed once real resource types exists.
pub struct TestResourceProc {}
impl ResourceProcessor for TestResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(TestResource {
            content: String::from("default content"),
            build_deps: vec![],
        })
    }

    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<AssetPathId> {
        resource
            .as_any()
            .downcast_ref::<TestResource>()
            .unwrap()
            .build_deps
            .clone()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.as_any().downcast_ref::<TestResource>().unwrap();
        let mut nbytes = 0;

        let content_bytes = resource.content.as_bytes();

        let bytes = content_bytes.len().to_ne_bytes();
        nbytes += bytes.len();
        writer.write_all(&bytes)?;
        nbytes += content_bytes.len();
        writer.write_all(content_bytes)?;

        let bytes = resource.build_deps.len().to_ne_bytes();
        nbytes += bytes.len();
        writer.write_all(&bytes)?;

        for dep in &resource.build_deps {
            let str = format!("{}", dep);
            let str = str.as_bytes();
            let bytes = str.len().to_ne_bytes();
            writer.write_all(&bytes)?;
            nbytes += bytes.len();
            writer.write_all(str)?;
            nbytes += str.len();
        }

        Ok(nbytes)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let mut resource = self.new_resource();
        let mut res = resource
            .as_any_mut()
            .downcast_mut::<TestResource>()
            .unwrap();

        let mut buf = 0usize.to_ne_bytes();
        reader.read_exact(&mut buf[..])?;
        let length = usize::from_ne_bytes(buf);

        let mut buf = vec![0u8; length];
        reader.read_exact(&mut buf[..])?;
        res.content = String::from_utf8(buf).unwrap();

        let mut buf = res.build_deps.len().to_ne_bytes();
        reader.read_exact(&mut buf[..])?;
        let dep_count = usize::from_ne_bytes(buf);

        for _ in 0..dep_count {
            let mut nbytes = 0u64.to_ne_bytes();
            reader.read_exact(&mut nbytes[..])?;
            let mut buf = vec![0u8; usize::from_ne_bytes(nbytes)];
            reader.read_exact(&mut buf)?;
            res.build_deps
                .push(AssetPathId::from_str(std::str::from_utf8(&buf).unwrap()).unwrap());
        }

        Ok(resource)
    }
}
