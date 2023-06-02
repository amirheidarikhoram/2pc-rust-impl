#![allow(dead_code)]

use core_2pc::{Transaction, TransactionState};

#[derive(Debug)]
pub struct TransactionPeer {
    pub id: String,
    pub transaction: Transaction,
}

#[derive(Debug)]
pub struct MultiPeerTransaction {
    pub id: String,
    pub tx_peers: Vec<TransactionPeer>,
    pub state: TransactionState,
    pub tx: Transaction,
}

impl MultiPeerTransaction {
    pub fn new(tx_peers: Vec<TransactionPeer>, tx: Transaction) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tx_peers,
            state: TransactionState::Begin,
            tx,
        }
    }
}
