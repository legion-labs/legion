use convert_case::{Case, Casing};

#[derive(Debug, Default, Clone)]
pub struct OpenAPIPath(Vec<String>);

impl OpenAPIPath {
    pub fn push(&mut self, s: impl Into<String>) {
        self.0.push(s.into());
    }

    pub fn to_pascal_case(&self) -> String {
        self.0.join("_").to_case(Case::Pascal)
    }
}

impl std::fmt::Display for OpenAPIPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined = self.0.join("/");
        write!(f, "{}", joined)
    }
}

impl From<&str> for OpenAPIPath {
    fn from(s: &str) -> Self {
        let parts = s.split('/').map(ToOwned::to_owned).collect();
        Self(parts)
    }
}
