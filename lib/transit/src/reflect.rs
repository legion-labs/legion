use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub name: &'static str,
    pub type_name: &'static str,
    pub offset: usize,
    pub size: usize,
    pub is_reference: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDefinedType {
    pub name: &'static str,
    pub size: usize,
    pub members: Vec<Member>,
}

pub trait Reflect {
    fn reflect() -> UserDefinedType;
}
