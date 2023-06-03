use serde::{Deserialize, Serialize};

use crate::Command;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Message {
    Begin((Command, String, String)), // command, mpt id, peer id
    Accept(String),                   // peer id
    Reject(String),                   // peer id on coordinator and mpt id on peer
    Commit(String),                   // mpt id
    Rollback(String),                 // mpt id
    Done(String),                     // mpt id
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
