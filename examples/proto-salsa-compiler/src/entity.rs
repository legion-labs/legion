use crate::inputs::Inputs;

#[salsa::query_group(EntityStorage)]
pub trait EntityCompiler: Inputs {
    fn compile_entity(&self) -> String;
}

pub fn compile_entity(_db: &dyn EntityCompiler) -> String {
    "Entity".to_owned()
}
