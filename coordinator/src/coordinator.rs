#![allow(dead_code)]

use std::{
    cell::RefCell,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread::{spawn, JoinHandle},
    time::Duration,
};

use core_2pc::{Command, Message, Transaction, TransactionState};

use crate::{
    peer::Peer,
    transaction::{MultiPeerTransaction, TransactionPeer},
};

type ArcOpMPT = Arc<Mutex<MultiPeerTransaction>>;

pub struct Coordinator {
    listener: TcpListener,
    messages: Vec<Message>,
    jh: JoinHandle<()>,
    peers: Arc<Mutex<Vec<Peer>>>,
    mp_transaction: ArcOpMPT,
    committing: bool,
}

impl Coordinator {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        let self_listener = listener.try_clone().unwrap();

        let peers = Arc::new(Mutex::new(Vec::new()));
        let peers_clone = Arc::clone(&peers);

        let mp_transactions: ArcOpMPT = Arc::new(Mutex::new(MultiPeerTransaction {
            id: format!("NONE"),
            tx_peers: vec![],
            state: TransactionState::Reject,
            tx: Transaction {
                command: Command {
                    query: format!("NONE"),
                    args: vec![],
                },
                state: TransactionState::Reject,
            },
        }));
        let mp_transactions_clone = Arc::clone(&mp_transactions);

        let jh = spawn(move || {
            let (tx, rx) = mpsc::channel::<Message>();

            spawn(move || handle_listener(listener, tx, peers_clone));

            let mp_transactions_clone = Arc::clone(&mp_transactions_clone);
            spawn(move || loop {
                let msg = rx.recv().unwrap();

                Self::handle_message(msg, Arc::clone(&mp_transactions_clone));
            });
        });

        Self {
            listener: self_listener,
            messages: Vec::new(),
            jh,
            peers,
            mp_transaction: mp_transactions,
            committing: false,
        }
    }

    pub fn handle_message(message: Message, mp_transactions: ArcOpMPT) {
        // We just update our state here due to the fact that there is no direct access to Coordinator state
        // so we just update mpt and on the other side we will decide to what to do based on the state.
        println!("Received: {:?}", message);

        match message {
            Message::Accept(_, peer_id) => {
                let mpts = mp_transactions.lock().unwrap();

                let peer = mpts.tx_peers.iter().find(|p| p.id == peer_id);

                if let Some(peer) = peer {
                    peer.update_state(TransactionState::Accept)
                }
            }
            Message::Reject(_, peer_id) => {
                let mpts = mp_transactions.lock().unwrap();

                if let Some(peer_id) = peer_id {
                    let peer = mpts.tx_peers.iter().find(|p| p.id == peer_id);

                    if let Some(peer) = peer {
                        peer.update_state(TransactionState::Reject)
                    }
                } else {
                    println!("Error: Coordinator got a reject message without peer id")
                }
            }
            Message::Done(peer_id) => {
                let mpts = mp_transactions.lock().unwrap();

                let peer = mpts.tx_peers.iter().find(|p| p.id == peer_id);

                if let Some(peer) = peer {
                    peer.update_state(TransactionState::Commit)
                }
            }
            _ => {}
        }
    }

    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<usize, String> {
        let tx_peers: Vec<TransactionPeer> = self
            .peers
            .lock()
            .unwrap()
            .iter()
            .map(|peer| TransactionPeer {
                id: peer.id.clone(),
                transaction: RefCell::new(transaction.clone()),
            })
            .collect();

        {
            let mut mp_transaction = self.mp_transaction.lock().unwrap();
            mp_transaction.tx_peers = tx_peers.clone();
            mp_transaction.tx = transaction.clone();
        }

        {
            let peer_ids;
            {
                let mp_transaction = self.mp_transaction.lock().unwrap();
                peer_ids = mp_transaction
                    .tx_peers
                    .iter()
                    .map(|p| p.id.clone())
                    .collect::<Vec<String>>();
            }
            self.broadcast_mpt_start(peer_ids, transaction.command.clone());
        }

        let mut tx_commit_check = 0;
        let mut tx_accept_check = 0;

        loop {
            let tx_peers;
            let peer_ids;
            {
                let mp_transaction = self.mp_transaction.lock().unwrap();
                tx_peers = mp_transaction.tx_peers.clone();
                peer_ids = mp_transaction
                    .tx_peers
                    .iter()
                    .map(|p| p.id.clone())
                    .collect::<Vec<String>>();
            }

            // check if transaction is committed by every one
            for tx in tx_peers.iter() {
                match tx.transaction.borrow().state {
                    TransactionState::Commit => {
                        tx_commit_check += 1;
                    }
                    TransactionState::Reject => {
                        // FIXME: this is a temp fix, mpt id in reject should be deleted
                        self.broadcast_mpt(peer_ids, Message::Reject(format!("NONE"), None));

                        return Err(format!("One of peers rejected the transaction"));
                    }
                    TransactionState::Accept => {
                        tx_accept_check += 1;
                    }
                    _ => {}
                }
            }

            if tx_accept_check == tx_peers.len() {
                if !self.committing {
                    self.broadcast_mpt(peer_ids, Message::Commit(format!("NONE")));
                }

                self.committing = true;
                continue;
            }

            if tx_commit_check == tx_peers.len() {
                self.committing = false;
                return Ok(tx_commit_check);
            }

            tx_commit_check = 0;
        }
    }

    // FIXME: remove mpt from args
    fn broadcast_mpt(&mut self, peer_ids: Vec<String>, message: Message) {
        let mut peers = self.peers.lock().unwrap();

        for peer_id in peer_ids {
            let peer = peers.iter_mut().find(|peer| peer.id == peer_id).unwrap();
            peer.stream
                .write(message.to_binary().unwrap().as_slice())
                .unwrap();
        }
    }

    fn broadcast_mpt_start(&mut self, peer_ids: Vec<String>, command: Command) {
        let mut peers = self.peers.lock().unwrap();

        for peer_id in peer_ids {
            let peer = peers.iter_mut().find(|peer| peer.id == peer_id).unwrap();
            peer.stream
                .write(
                    // FIXME: remove mpt id
                    Message::Begin(command.clone(), format!("NONE"), peer_id)
                        .to_binary()
                        .unwrap()
                        .as_slice(),
                )
                .unwrap();
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
