use serde::{Deserialize, Serialize};

use crate::Command;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Message {
    Begin(Command, String, String), // command, mpt_id, peer_id
    Accept(String, String),         // mpt_id, peer_id
    Reject(String, Option<String>), // mpt_id, peer_id?
    Commit(String),                 // mpt_id
    Rollback(String),               // mpt_id
    Done(String),                   // mpt_id
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
