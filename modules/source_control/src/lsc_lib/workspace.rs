use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Workspace {
    pub id: String, //a file lock will contain the workspace id
    pub repository: String,
    pub owner: String,
}
