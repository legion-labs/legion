use crate::{Locale, Platform, Target};

#[salsa::query_group(InputsStorage)]
pub trait Inputs {
    #[salsa::input]
    fn input_file(&self, name: String) -> String;

    #[salsa::input]
    fn platform(&self) -> Platform;

    #[salsa::input]
    fn target(&self) -> Target;

    #[salsa::input]
    fn locale(&self) -> Locale;
}
