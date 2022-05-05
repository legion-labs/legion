use fluent_syntax::parser::ParserError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid glob pattern: {0}")]
    InvalidGlobPattern(#[from] glob::PatternError),

    #[error("Entry is not a message entry")]
    EntryNotMessage,

    #[error("Invalid glob: {0}")]
    InvalidGlob(#[from] glob::GlobError),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Failed to parse an FTL resource: {0:?}")]
    FluentParse(Vec<ParserError>),

    #[error(transparent)]
    AskamaRender(#[from] askama::Error),

    #[error("Out dir exists but is not a directory")]
    OutDirNotDir,
}

pub type Result<T> = std::result::Result<T, Error>;
