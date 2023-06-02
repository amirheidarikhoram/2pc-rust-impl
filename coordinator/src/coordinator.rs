use std::{
    collections::HashMap,
    io::Read,
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread::{spawn, JoinHandle},
    time::Duration,
};

use core_2pc::Message;

use crate::{peer::Peer, transaction::MultiPeerTransaction};

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
