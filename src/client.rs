use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, ToSocketAddrs};

pub const SEND: bool = true;
pub const RECEIVE: bool = false;

#[derive(Debug)]
pub(crate) struct Account {
    pub username: String,
    password: String,
}


#[derive(Debug)]
pub struct Client {
    pub(crate) account: Account,
    socket: TcpStream,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub receiver: String,
    pub from: String,
    pub _type: bool,
    pub data: Vec<u8>,
}

impl Message {
    pub fn new(name: &str, data: &[u8], _type: bool, from: String) -> Self {
        let name = name.eq("server").then_some("server").or(Some(name)).unwrap();
        Self {
            receiver: name.to_string(),
            from,
            _type,
            data: Vec::from(data),
        }
    }
}

impl Client {
    pub async fn connect(username: impl ToString,
                         password: impl ToString,
                         addr: impl ToSocketAddrs) -> io::Result<Self> {
        let mut socket = TcpStream::connect(addr).await?;
        //socket.write(username.to_string().as_bytes()).await?;

        Ok(Self {
            account: Account {
                username: username.to_string(),
                password: password.to_string(),
            },
            socket,
        })
    }
}

impl Client {
    pub async fn send(&mut self, data: &str, name: &str, user: String) -> io::Result<usize> {
        let msg = Message::new(name, data.as_bytes(), SEND, self.account.username.clone());
        let ser = serde_json::to_string(&msg)?;
        let count = self.socket.write(ser.as_bytes()).await?;
        println!("[client] {} send {} to {},{} bytes in total", msg.from, data, msg.receiver, count);
        Ok(count)
    }

    pub async fn receive(&self) {
        println!("2333");
    }
}