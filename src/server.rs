use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::client::{Message, RECEIVE};

pub struct Server {
    listener: TcpListener,
    clients: Arc<RwLock<HashMap<SocketAddr, String>>>,
    msg_buffer: Arc<RwLock<HashMap<String, Vec<Vec<u8>>>>>,
}

impl<'a> Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            clients: Arc::new(RwLock::new(HashMap::new())),
            msg_buffer: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&mut self) -> io::Result<()> {
        println!("server start at {}", self.listener.local_addr().unwrap());
        loop {
            let (mut socket, addr) = self.listener.accept().await?;
            println!("connection from: {}", addr);

            let clients = self.clients.clone();

            let thread: JoinHandle<_> = tokio::spawn(async move {
                loop {
                    let mut buf = vec![0; 1024];
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("shutdown {}", addr);
                            break;
                        }
                        Ok(t) => {
                            buf.resize(t, 0u8);
                            let msg: Result<Message, serde_json::Error> = serde_json::from_slice(buf.as_slice());
                            let msg = msg.unwrap();

                            let receiver = msg.receiver;
                            let data = msg.data;
                            let _type = msg._type;

                            let mut client = clients.write().await;
                            match client.get(&addr) {
                                Some(username) => {
                                    println!("[server] message from {},to {}", username, receiver);
                                    if receiver.eq(&String::from("server")) {
                                        println!("{}", String::from_utf8(data).unwrap());
                                    } else {
                                        println!("{}", String::from_utf8(data).unwrap());
                                    }
                                }
                                None => {
                                    let s = msg.from.clone();
                                    println!("username: {}", s);
                                    println!("[server] message from {},to {}", s, receiver);
                                    if receiver.eq(&String::from("server")) {
                                        println!("{}", String::from_utf8(data).unwrap());
                                    } else {
                                        println!("{}", String::from_utf8(data).unwrap());
                                    }
                                    client.insert(addr, s.replace("\n", ""));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            return;
                        }
                    }
                }
            });
        }
    }

    pub async fn serve(&mut self, msg: &Message, socket: &mut TcpStream, addr: &SocketAddr) {
        let clients = self.clients.clone();
        let buffers = self.msg_buffer.clone();
        if msg._type == RECEIVE {
            let buffer = buffers.write().await;
            match buffer.get(msg.from.as_str()) {
                None => {
                    let _ = socket.write("no msg send".as_bytes()).await;
                }
                Some(x) => {
                    for msg_line in x {
                        let _ = socket.write(msg_line.as_slice()).await;
                    }
                }
            }
        } else {
            let mut buffer = self.msg_buffer.write().await;
            let mut client = clients.write().await;
            client.get(addr).map(|user| {}).or_else(|| {
                client.insert(*addr, msg.from.replace("\n", ""));
                buffer.get_mut(msg.receiver.as_str()).map(|msg_buffer: &mut Vec<Vec<u8>>| {
                    msg_buffer.push(msg.data.clone());
                }).or_else(|| {
                    buffer.insert(msg.receiver.clone(), vec![msg.data.clone()]);
                    None
                });
                None
            });
        }
    }
}
//{"receiver":"tes","from":"absd","_type":true,"data":[116,101,115,116]}