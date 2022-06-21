use crate::inputs::Inputs;

#[salsa::query_group(ResiyrceStorage)]
pub trait ResourceCompiler: Inputs {
    fn compile_resource(&self, resource_path_id: String) -> String;
}

pub fn compile_resource(_db: &dyn ResourceCompiler, _resource_path_id: String) -> String {
    "".to_owned()
}
