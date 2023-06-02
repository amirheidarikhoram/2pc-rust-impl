use std::net::TcpStream;

#[derive(Debug)]
pub struct Peer {
    pub id: String,
    pub stream: TcpStream,
}
