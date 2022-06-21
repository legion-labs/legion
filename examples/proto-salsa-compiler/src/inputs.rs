#[salsa::query_group(InputsStorage)]
pub trait Inputs {
    #[salsa::input]
    fn input_file(&self, name: String) -> String;
}
