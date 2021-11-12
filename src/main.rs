#![feature(bool_to_option)]

use std::error::Error;

use clap::clap_app;

use crate::client::Client;
use crate::server::Server;

mod server;
mod client;


const DEFAULT_ADDR: &'static str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = clap_app!(simple_client =>
        (version: "0.0.1")
        (@arg SERVER: -s --server "server start")
        (@arg ADDRESS: -a --address +takes_value "address")
        (@arg USERNAME: -u --username +takes_value "username")
        (@arg PASSWORD: -p --password +takes_value "password")
        (@arg SEND:-S --send + takes_value "send message")
        (@arg RECEIVE:-r --receive "receive message")
        (@arg RECEIVER:-R --receiver + takes_value "receiver")
    ).get_matches();
    if args.is_present("SERVER") {
        let addr = match args.value_of("ADDRESS") {
            Some(addr) => addr,
            None => DEFAULT_ADDR
        };
        let mut s = Server::bind(addr).await?;
        s.start().await?;
    } else {
        let addr = match args.value_of("ADDRESS") {
            Some(addr) => addr,
            None => DEFAULT_ADDR
        };
        let username = args.value_of("USERNAME").unwrap();
        let password = args.value_of("PASSWORD").unwrap();
        let send_msg = args.value_of("SEND").unwrap();
        let receiver = args.value_of("RECEIVER");

        assert!(args.is_present("SEND") || args.is_present("RECEIVE"));

        if args.is_present("SEND") {
            let mut c = Client::connect(username, password, addr).await?;
            //println!("welcome {}!", username);
            let recv;
            if let Some(x) = receiver {
                recv = x;
            } else {
                recv = "server";
            }
            let count = c.send(send_msg, recv, c.account.username.clone()).await?;
            //println!("[client] {} send {} bytes to {}", username, count, receiver.unwrap());
        } else {
            let mut c = Client::connect(username, password, addr).await?;
            c.receive();
        }
    };

    Ok(())
}