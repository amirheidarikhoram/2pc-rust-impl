use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

use core_2pc::{convert_args, Message, Transaction, TransactionState};
use deadpool_postgres::Pool;
use tokio::{
    sync::{mpsc, Mutex},
    task::spawn,
};

pub struct Element {
    pub stream: Arc<Mutex<TcpStream>>,
}

impl Element {
    pub async fn new() -> Self {
        let stream = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").unwrap()));

        stream
            .lock()
            .await
            .set_read_timeout(Some(Duration::new(0, 1)))
            .expect("Couldn't set read timeout on stream");

        Self { stream }
    }

    pub async fn run(self, pool: Arc<Mutex<Pool>>) {
        const FRAME_SIZE: usize = 1024;

        let (tx, rx) = mpsc::channel::<Message>(1);
        let handler_stream = Arc::clone(&self.stream);
        let pool = Arc::clone(&pool);
        spawn(async move { handle_channel_messages(rx, handler_stream, pool).await });

        let stream = self.stream.clone();
        loop {
            let mut buf = [0; FRAME_SIZE];

            let mut stream = stream.lock().await;

            if let Ok(_) = stream.read(&mut buf) {
                if let Ok(message) = Message::from_binary(buf.as_slice()) {
                    tx.send(message).await.unwrap();
                } else {
                    eprintln!("error: failed to deserialize message")
                }
            }
        }
    }
}

async fn handle_channel_messages(
    mut rx: mpsc::Receiver<Message>,
    stream: Arc<Mutex<TcpStream>>,
    pool: Arc<Mutex<Pool>>,
) {
    loop {
        let mut peer_transaction: Option<Transaction> = None;
        let mut peer_id: Option<String> = None;

        loop {
            if let Some(message) = rx.recv().await {
                let transaction = peer_transaction.clone();
                let id = peer_id.clone();

                match message {
                    Message::Begin(command, mpt_peer_id) => {
                        // Note that we don't check if there was any ongoing transaction, we just replace it.

                        let pool = pool.lock().await;
                        let mut client = pool.get().await.unwrap();
                        let db_transaction = client.transaction().await.unwrap();

                        let params = convert_args(command.args.iter());
                        if let Err(err) = db_transaction.execute(&command.query, &params).await {
                            println!("Error: {:?}", err);

                            update_transaction_state(&stream, Message::Reject(Some(mpt_peer_id)))
                                .await;
                            continue;
                        }

                        peer_transaction = Some(Transaction {
                            state: TransactionState::Accept,
                            command: command.clone(),
                        });
                        peer_id = Some(mpt_peer_id.clone());
                        update_transaction_state(&stream, Message::Accept(mpt_peer_id)).await;
                    }
                    Message::Commit() => {
                        if transaction.is_none() || id.is_none() {
                            break;
                        }

                        let pool = pool.lock().await;
                        let mut client = pool.get().await.unwrap();
                        let db_transaction = client.transaction().await.unwrap();

                        let command = peer_transaction.clone().unwrap().command;

                        let params = convert_args(command.args.iter());
                        if let Err(err) = db_transaction.execute(&command.query, &params).await {
                            println!("Error: {:?}", err);

                            update_transaction_state(&stream, Message::Reject(id)).await;
                            continue;
                        }

                        db_transaction.commit().await.unwrap();
                        update_transaction_state(&stream, Message::Done(peer_id.clone().unwrap()))
                            .await;
                    }
                    Message::Reject(..) => {
                        if !peer_transaction.is_none() {
                            peer_transaction = None;
                        }
                    }
                    Message::Done(_) => {
                        if !peer_transaction.is_none() {
                            peer_transaction = None;
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }
}

async fn update_transaction_state(stream: &Arc<Mutex<TcpStream>>, state: Message) {
    let mut stream = stream.lock().await;

    stream.write(state.to_binary().unwrap().as_slice()).unwrap();
}
