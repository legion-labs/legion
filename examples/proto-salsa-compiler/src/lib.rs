mod inputs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAddr(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Platform {
    PS5,
    XSX,
    XB1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    Client,
    Server,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Locale {
    English,
    French,
    Spanish,
    Japenese,
}
