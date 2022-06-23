#[salsa::query_group(InputsStorage)]
pub trait Inputs {
    #[salsa::input]
    fn read(&self, name: String) -> String;
}
