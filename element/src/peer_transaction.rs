use core_2pc::Transaction;

#[derive(Clone)]
pub struct PeerTransaction {
    pub transaction: Transaction,
    pub id: String,
}
