use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Command {
    pub query: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn to_binary(&self) -> Result<Vec<u8>, String> {
        match bincode::serialize(&self) {
            Ok(data) => Ok(data),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn from_binary(data: &[u8]) -> Result<Self, String> {
        match bincode::deserialize(data) {
            Ok(data) => Ok(data),
            Err(e) => Err(e.to_string()),
        }
    }
}
