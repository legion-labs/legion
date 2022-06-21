use crate::inputs::Inputs;

#[salsa::query_group(ResourceStorage)]
pub trait ResourceCompiler: Inputs {
    fn compile_resource(&self, resource_path_id: String) -> String;
}

pub fn compile_resource(_db: &dyn ResourceCompiler, resource_path_id: String) -> String {
    "".to_owned()
}
