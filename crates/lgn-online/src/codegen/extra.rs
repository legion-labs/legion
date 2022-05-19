use http::HeaderMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extra {
    pub headers: HeaderMap,
}
