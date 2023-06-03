use serde::{Deserialize, Serialize};

use crate::Command;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Message {
    Begin((Command, String)),
    Accept(String),
    Reject(String),
    Commit(String),
    Rollback(String),
    Done(String),
}

impl Message {
    pub fn to_binary(&self) -> Result<Vec<u8>, String> {
        match bincode::serialize(&self) {
            Ok(data) => Ok(data),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn from_binary(data: &[u8]) -> Result<Message, String> {
        match bincode::deserialize(data) {
            Ok(data) => Ok(data),
            Err(e) => Err(e.to_string()),
        }
    }
}
