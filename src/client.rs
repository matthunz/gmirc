extern crate reqwest;
extern crate hyper;
extern crate serde_json;

use self::hyper::header::{ContentType};
use serde_json::Value;
use std::sync::mpsc::Sender;

pub fn post_json(url: &str, body: Value) -> Option<Value> {
    let client = reqwest::Client::new().unwrap();
    let res = client.post(url)
        .unwrap()
        .header(ContentType::json())
        .body(body.to_string())
        .send();

    match res {
        Ok(res) => {
            let mut res = res;
            let json: Value = res.json().expect("GroupMe returned invalid json");
            Some(json)
        }
        Err(_) => None
    }
}

fn post_faye(body: Value) -> Option<Value> {
    post_json("https://push.groupme.com/faye", body)
}

pub struct Client {
    client_id: String,
    tx: Sender<Value>,
    user_id: String
}

impl Client {
    pub fn new(tx: Sender<Value>) -> Client {
        Client {
            client_id: String::new(),
            user_id: String::new(),
            tx: tx
        }
    }

    pub fn run(&mut self) {
        self.user_id = self.get_user_id();
        self.client_id = self.get_client_id();

        self.get_groups();
        self.subscribe_user();
        self.poll_data();
    }

    fn get_user_id(&mut self) -> String {
        let url = format!("https://api.groupme.com/v3/users/me?token={}", ::token::TOKEN);
        let mut res = reqwest::get(&url).expect("Error connecting to GroupMe");

        let json: Value = res.json().expect("GroupMe returned invalid json");
        json["response"]["id"].as_str().unwrap().to_owned()
    }

    fn get_client_id(&mut self) -> String {
        let body = json!({
            "channel": "/meta/handshake",
            "version": "1.0",
            "supportedConnectionTypes": ["long-polling"],
            "id": "1"
        });

        post_faye(body).unwrap()[0]["clientId"].as_str().expect("Recieved invalid response").to_owned()
    }

    fn get_groups(&mut self) {
        let url = format!("https://api.groupme.com/v3/groups?token={}", ::token::TOKEN);
        let json: Value = reqwest::get(&url)
            .expect("Error getting groups from groupme")
            .json()
            .expect("Groupme returned invalid json");

        for group in json["response"].as_array().unwrap() {
            self.tx.send(json!({
                "type": "group",
                "id": group["group_id"].as_str().unwrap(),
                "name": "#".to_owned() + &group["name"].as_str().unwrap().replace(" ", "_")
            })).expect("Error sending to tx pipe");
        }

        self.tx.send(json!({
            "type": "welcome"
        })).expect("Error sending to tx pipe");
    }

    fn subscribe_user(&mut self) {
        let body = json!({
            "channel": "/meta/subscribe",
            "clientId": self.client_id,
            "subscription": format!("/user/{}", self.user_id),
            "id": "2",
            "ext":
            {
                "access_token": ::token::TOKEN
            }
        });

        if !post_faye(body).unwrap()[0]["successful"].as_bool().unwrap() {
            panic!("User subscription failed")
        }
    }

    fn poll_data(&mut self) {
        loop {
            let body = json!({
                "channel": "/meta/connect",
                "clientId": self.client_id,
                "connectionType": "long-polling",
                "id":"3"
            });

            match post_faye(body) {
                Some(json) => {
                    let msg = json!({
                        "type": "event",
                        "subject": json[1]["data"]["subject"]
                    });
                    self.tx.send(msg).expect("Error sending tx");
                }
                None => { continue; }
            };
        }
    }
}
