#[macro_use]
extern crate serde_json;

mod connection;
mod client;
mod token;

use std::net::TcpListener;
use std::thread;
use connection::Connection;
use client::Client;

fn main() {
    let mut client = Client::new();
    client.run();

    let listener = TcpListener::bind("127.0.0.1:6667").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut conn = Connection::new(stream.unwrap());
            conn.run();
        });
    }
}
