use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6667").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut client = Client::new(stream.unwrap());
            client.run();
        });
    }
}

struct Client {
    stream: TcpStream,
    nick: String
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        Client { stream: stream, nick: String::new() }
    }

    fn run(&mut self) {
        loop {
            let mut buffer = [0; 512];
            self.stream.read(&mut buffer).unwrap();

            let line = String::from_utf8(buffer.to_vec()).unwrap();
            self.parse_command(&line);

            let msg = format!(":gmirc 001 {0} :Welcome!\r {0}", self.nick);
            if self.stream.write(msg.as_bytes()).is_err() {
                break;
            }
        }
        self.nick = String::from("test2")
    }

    fn parse_command(&mut self, line: &str) {
        if line.contains("NICK") {
            self.nick = String::from(line.split_at(5).1)
        }
    }
}
