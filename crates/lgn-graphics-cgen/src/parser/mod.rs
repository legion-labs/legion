use std::path::PathBuf;

mod parser_rune;
pub(crate) use parser_rune::from_rune;

use crate::db::Model;

pub(crate) struct ParsingResult {
    pub input_dependencies: Vec<PathBuf>,
    pub model: Model,
}
