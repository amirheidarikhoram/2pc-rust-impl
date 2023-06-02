use std::{
    io::Read,
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

use core_2pc::Message;
use deadpool_postgres::Pool;
use tokio::{sync::mpsc, task::spawn};

pub struct Element {
    pub stream: Arc<Mutex<TcpStream>>,
}

impl Element {
    pub fn new() -> Self {
        let stream = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").unwrap()));

        stream
            .lock()
            .unwrap()
            .set_read_timeout(Some(Duration::new(0, 1)))
            .expect("Couldn't set read timeout on stream");

        Self { stream }
    }

    pub async fn run(self, pool: Arc<Mutex<Pool>>) {
        const FRAME_SIZE: usize = 1024;

        let (tx, rx) = mpsc::channel::<Message>(1);

        let handler_stream = Arc::clone(&self.stream);
        spawn(async move { move || handle_channel_messages(rx, handler_stream, pool) });

        let stream = self.stream.clone();
        loop {
            let mut buf = [0; FRAME_SIZE];

            let mut stream = stream.lock().unwrap();
            stream.read(&mut buf).unwrap();
            if let Ok(message) = Message::from_binary(buf.as_slice()) {
                tx.send(message).await.unwrap();
            } else {
                eprintln!("error: failed to deserialize message")
            }
        }
    }
}

async fn handle_channel_messages(
    mut rx: mpsc::Receiver<Message>,
    stream: Arc<Mutex<TcpStream>>,
    pool: Arc<Mutex<Pool>>,
) {
    todo!()
}
