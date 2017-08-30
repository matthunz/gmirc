use std::collections::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use client;
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
                "welcome" => {
                    let msg = format!("001 {0} :Welcome! {0}[!{1}@{2}]",
                                      self.nick, self.user, self.hostname);
                    self.send_command(&msg);

                }
                "irc" => {
                    let msg = json["msg"].as_str().unwrap();
                    self.parse_command(&msg);
                }
                "user" => {
                    self.nick = json["name"].as_str().unwrap().to_owned().replace(" ", "_")
                }
                "group" => {
                    let id =  json["id"].as_str().unwrap().to_owned();
                    let name = json["name"].as_str().unwrap().to_owned();

                    self.channels.insert(name, id);
                }
                _ => {
                    let subject = &json["subject"];
                    let group_id = subject["group_id"].as_str().unwrap_or_default();

                    let channels = self.channels.clone();
                    let channel = channels.iter().filter(|c| {
                        let &(_name, id) = c;
                        id == group_id
                    }).next();

                    if let Some(channel) = channel {
                        let name = subject["name"]
                            .as_str()
                            .and_then(|n| { Some(n.replace(" ", "_" )) })
                            .unwrap_or_default();
                        let text = subject["text"].as_str().unwrap_or_default();

                        let msg = format!(":{} PRIVMSG {} :{}", name, channel.0, text);
                        self.send_message(&msg);

                    }
                }
            }
        }
    }

    fn parse_command(&mut self, line: &str) {
        if line.contains("USER") {
            let tokens:  Vec<&str> = line.split(" ").collect();

            self.user = tokens[1].to_owned();
            self.hostname = tokens[2].to_owned();

        } else if line.contains("JOIN") {
            let channel_name = line.split_at(5).1;

            match self.channels.clone().get(channel_name) {
                Some(_) => {
                    self.joined.push(channel_name.to_owned());

                    let msg = format!("332 {0} {1} :topic", self.nick, channel_name);
                    self.send_command(&msg);

                    let msg = format!(":{0}!{1} JOIN {2}",
                                      self.nick, self.hostname, channel_name);
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

        } else if line.contains("PRIVMSG") {
            let mut tokens: Vec<&str> = line.split_terminator(' ').collect();
            tokens.remove(0);

            match self.channels.clone().get(tokens.remove(0)) {
                Some(channel_id) => {
                    let mut msg = tokens.join(" ");
                    msg.remove(0); // remove :

                    let url = format!("{}/groups/{}/messages?token={}", ::BASE_URL, channel_id, ::token::TOKEN);
                    let json = json!({
                        "message": {
                            "text": msg
                        }
                    });

                    if let Some(_) = client::post_json(&url, json) {
                        self.send_command("404 :Sending message to GroupMe failed");
                    };
                }
                None => {
                    self.send_command("404 :Channel does not exist");
                }
            } 
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
