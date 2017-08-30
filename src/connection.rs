use std::collections::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use client::Client;

pub struct Connection {
    stream: TcpStream,
    nick: String,
    user: String,
    hostname: String,
    channels: HashMap<String, String>,
    joined: Vec<String>
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: stream,
            nick: String::new(),
            user: String::new(),
            hostname: String::new(),
            channels: HashMap::new(),
            joined: Vec::new()
        }
    }

    pub fn run(&mut self) {
        let mut buffer = BufReader::new(self.stream.try_clone().unwrap());
        let (tx, rx) = mpsc::channel();
        let client_tx = tx.clone();

        thread::spawn(move || {
            let mut client = Client::new(client_tx);
            client.run();
        });


        thread::spawn(move || {
            loop {
                let mut msg = String::new();

                match buffer.read_line(&mut msg) {
                    Ok(bytes) => {
                        if bytes == 0 { break; }

                        // remove \r\n
                        let new_len = msg.len() - 2;
                        msg.truncate(new_len);

                        println!("Received data ({} bytes): {}", bytes, msg);
                        let json = json!({
                            "type": "irc",
                            "msg": msg
                        });
                        tx.send(json).expect("Could not send to tx");
                    }
                    Err(_) => println!("Error receiving data")
                }
            }
        });

        loop {
            let json = rx.recv().expect("Could not read rx");
            match json["type"].as_str().unwrap() {
                "irc" => {
                    let msg = json["msg"].as_str().unwrap();
                    self.parse_command(&msg);
                }
                "group" => {
                    let id =  json["id"].as_str().unwrap().to_owned();
                    let name = json["name"].as_str().unwrap().to_owned();

                    self.channels.insert(id, name);
                }
                _ => {
                    let subject = &json["subject"];
                    let group_id = subject["group_id"].as_str().unwrap_or_default();

                    if let Some(group_name) = self.channels.clone().get(group_id) {
                        let name = subject["name"]
                            .as_str()
                            .and_then(|n| { Some(n.replace(" ", "_" )) })
                            .unwrap_or_default();
                        let text = subject["text"].as_str().unwrap_or_default();

                        let msg = format!(":{} PRIVMSG {} :{}", name, group_name, text);
                        self.send_message(&msg);
                    }
                }
            }
        }
    }

    fn parse_command(&mut self, line: &str) {
        if line.contains("NICK") {
            self.nick = line.split_at(5).1.to_owned();

        } else if line.contains("USER") {
            let tokens:  Vec<&str> = line.split(" ").collect();

            self.user = tokens[1].to_owned();
            self.hostname = tokens[2].to_owned();

            // welcome the newly defined user
            let msg = format!("001 {0} :Welcome! <{0}>[!<{1}>@<{2}>]",
                              self.nick, self.user, self.hostname);
            self.send_command(&msg);

        } else if line.contains("JOIN") {
            let channel_name = line.split_at(5).1.to_owned();

            let channels = self.channels.clone();
            let mut filter = channels.iter().filter(|c| {
                let &(id, name) = c;
                name == &channel_name
            });

            match filter.next() {
                Some(channel) => {
                    self.joined.push(channel.0.clone());

                    let msg = format!("332 {0} {1} :topic", self.nick, channel.1);
                    self.send_command(&msg);

                    let msg = format!(":{0}!{1} JOIN {2}",
                                      self.nick, self.hostname, channel.1);
                    self.send_message(&msg);
                }
                None => {
                    let msg = format!("403 :{}!{} {} :No such channel",
                                      self.nick, self.hostname, channel_name);
                    self.send_message(&msg);
                }
            }

        } else if line.contains("PING") {
            let msg = format!("PONG {}", line.split_at(5).1);
            self.send_message(&msg);
        }
    }

    fn send_message(&mut self, msg: &str) {
        let msg = format!("{}\r\n", msg);

        println!("Sent: {:?}", msg);
        self.stream.write(msg.as_bytes()).unwrap();
    }

    fn send_command(&mut self, msg: &str) {
        let msg = format!(":gmirc {}", msg);
        self.send_message(&msg);
    }
}
