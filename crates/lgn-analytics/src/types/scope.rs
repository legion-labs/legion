#[derive(Clone, PartialEq)]
pub struct ScopeDesc {
    pub name: String,
    pub filename: String,
    pub line: u32,
    pub hash: u32,
}
