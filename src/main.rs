use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6667").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            handle_client(stream.unwrap());
        });
    }
}

fn handle_client(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        if stream.write("test\r\n".as_bytes()).is_err() {
            break;
        }

        println!("{}", String::from_utf8_lossy(&buffer[..]))
    }
}
