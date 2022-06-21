use crate::inputs::Inputs;

#[salsa::query_group(MaterialStorage)]
pub trait MaterialCompiler: Inputs {
    fn compile_material(&self) -> String;
}

pub fn compile_material(_db: &dyn MaterialCompiler) -> String {
    "Compiler".to_owned()
}
