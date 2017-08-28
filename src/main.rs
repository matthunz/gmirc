use std::io::BufReader;
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
    nick: String,
    user: String,
    hostname: String
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        Client {
            stream: stream,
            nick: String::new(),
            user: String::new(),
            hostname: String::new()
        }
    }

    fn run(&mut self) {
        loop {
            let mut buffer = BufReader::new(self.stream.try_clone().unwrap());
            let mut msg = String::new();

            match buffer.read_line(&mut msg) {
                Ok(bytes) => {
                    if bytes == 0 { break; }

                    print!("Received data ({} bytes): {}", bytes, msg); 
                    self.parse_command(&msg);
                } 
                Err(_) => println!("Error receiving data")
            }

        }
    }

    fn parse_command(&mut self, line: &str) {
        if line.contains("NICK") {
            self.nick = line.split_at(5).1.to_owned();

            // remove \r\n
            let new_len = self.nick.len() - 2;
            self.nick.truncate(new_len);

        } else if line.contains("USER") {
            let tokens:  Vec<&str> = line.split(" ").collect();

            self.user = tokens[1].to_owned();
            self.hostname = tokens[2].to_owned();

            // welcome the newly defined user
            let msg = format!(":gmirc 001 {0} :Welcome! <{0}>[!<{1}>@<{2}>]\r\n",
                              self.nick, self.user, self.hostname);
            self.stream.write(msg.as_bytes()).unwrap();
        }
    }
}
