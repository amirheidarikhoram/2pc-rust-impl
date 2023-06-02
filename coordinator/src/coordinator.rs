#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread::{spawn, JoinHandle},
    time::Duration,
};

use core_2pc::{Message, Transaction, TransactionState};

use crate::{
    peer::Peer,
    transaction::{MultiPeerTransaction, TransactionPeer},
};

pub struct Coordinator {
    listener: TcpListener,
    messages: Vec<Message>,
    jh: JoinHandle<()>,
    peers: Arc<Mutex<Vec<Peer>>>,
    mp_transactions: Arc<Mutex<HashMap<String, MultiPeerTransaction>>>,
}

impl Coordinator {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        let self_listener = listener.try_clone().unwrap();

        let peers = Arc::new(Mutex::new(Vec::new()));
        let peers_clone = Arc::clone(&peers);

        let jh = spawn(move || {
            let (tx, rx) = mpsc::channel::<Message>();

            spawn(move || handle_listener(listener, tx, peers_clone));

            spawn(move || loop {
                let msg = rx.recv().unwrap();

                Self::handle_message(msg);
            });
        });

        Self {
            listener: self_listener,
            messages: Vec::new(),
            jh,
            peers,
            mp_transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn handle_message(message: Message) {
        println!("Received: {:?}", message);
    }

    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<usize, String> {
        let tx_peers = self
            .peers
            .lock()
            .unwrap()
            .iter()
            .map(|peer| TransactionPeer {
                id: peer.id.clone(),
                transaction: transaction.clone(),
            })
            .collect();

        let mp_transaction = MultiPeerTransaction::new(tx_peers, transaction);
        let mpt_id = mp_transaction.id.clone();

        self.broadcast_mpt(
            &mp_transaction,
            Message::Begin((mp_transaction.tx.clone().command, mpt_id.clone())),
        );

        self.mp_transactions
            .lock()
            .unwrap()
            .insert(mpt_id.clone(), mp_transaction);

        let mut tx_commit_check = 0;

        loop {
            let mp_transactions = self.mp_transactions.lock().unwrap();

            // check if transaction is committed by every one
            let mp_transaction = mp_transactions.get(&mpt_id).unwrap();
            for tx in mp_transaction.tx_peers.iter() {
                match tx.transaction.state {
                    TransactionState::Commit => {
                        tx_commit_check += 1;
                    }
                    TransactionState::Reject => {
                        // TODO: broadcast abort to all peers

                        return Err(format!("One of peers rejected the transaction"));
                    }
                    _ => {}
                }
            }

            if tx_commit_check == mp_transaction.tx_peers.len() {
                return Ok(tx_commit_check);
            }

            tx_commit_check = 0;
        }
    }

    fn broadcast_mpt(&mut self, mpt: &MultiPeerTransaction, message: Message) {
        let mut mtp_peer_ids = mpt.tx_peers.iter().map(|peer| {
            let id = peer.id.clone();
            id
        });

        let mut counter = self.peers.lock().unwrap().len();

        let mut peers = self.peers.lock().unwrap();

        while counter > 0 {
            if let Some(peer_id) = mtp_peer_ids.next() {
                let peer = peers.iter_mut().find(|peer| peer.id == peer_id).unwrap();
                peer.stream
                    .write(message.to_binary().unwrap().as_slice())
                    .unwrap();
            }
            counter -= 1;
        }
    }
}

fn handle_listener(listener: TcpListener, tx: mpsc::Sender<Message>, peers: Arc<Mutex<Vec<Peer>>>) {
    for stream in listener.incoming() {
        let tx = tx.clone();

        if let Ok(stream) = stream {
            let id = uuid::Uuid::new_v4().to_string();
            peers.lock().unwrap().push(Peer {
                id,
                stream: stream.try_clone().unwrap(),
            });
            spawn(|| {
                handle_stream(stream, tx);
            });
        }
    }
}

fn handle_stream(stream: TcpStream, tx: mpsc::Sender<Message>) {
    static FRAME_SIZE: usize = 1024;

    stream
        .set_read_timeout(Some(Duration::new(0, 1)))
        .expect("Couldn't set read timeout on stream");
    let stream = Arc::new(Mutex::new(stream));
    let read_stream = Arc::clone(&stream);

    let _connection_jh = spawn(move || {
        loop {
            let mut buf = vec![0u8; FRAME_SIZE];
            let mut read_stream = read_stream.lock().unwrap();
            match read_stream.read(&mut buf) {
                Ok(_) => {
                    // successful read, send message to  coordinator
                    if let Ok(message) = Message::from_binary(buf.as_slice()) {
                        tx.send(message).expect("Couldn't send message");
                    }
                }
                Err(_) => {}
            }
        }
    });
}
