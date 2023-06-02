use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TransactionState {
    Begin,
    Reject,
    Accept,
    Commit,
    Rollback,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Transaction {
    pub state: TransactionState,
    pub command: String,
}

impl Transaction {
    pub fn new(command: String) -> Self {
        Self {
            state: TransactionState::Begin,
            command,
        }
    }

    pub fn to_binary(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_binary(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}
