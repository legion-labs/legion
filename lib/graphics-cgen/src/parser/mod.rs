use std::path::PathBuf;

mod parser_syn;
pub(crate) use parser_syn::from_syn;

use crate::model::Model;

pub(crate) struct ParsingResult {
    pub input_dependencies: Vec<PathBuf>,
    pub model: Model,
}
