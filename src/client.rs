extern crate reqwest;
extern crate hyper;
extern crate serde_json;

use self::hyper::header::{ContentType};
use serde_json::{Value};

pub struct Client {
    client: reqwest::Client,
    client_id: String,
    user_id: String
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest::Client::new().unwrap(),
            client_id: String::new(),
            user_id: String::new()
        }
    }

    pub fn run(&mut self) {
        self.user_id = self.get_user_id();
        self.client_id = self.get_client_id();
        self.subscribe_user();

        loop {
            self.poll_data();
        }
    }

    fn post_json(&mut self, body: Value) -> Value {
        let mut res = self.client.post("https://push.groupme.com/faye")
            .unwrap()
            .header(ContentType::json())
            .body(body.to_string())
            .send()
            .unwrap();

        let json: Value = res.json().expect("GroupMe returned invalid json");
        json
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

        self.post_json(body)[0]["clientId"].as_str().expect("Recieved invalid response").to_owned()
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

        if !self.post_json(body)[0]["successful"].as_bool().unwrap() {
            panic!("User subscription failed")
        }
    }

    fn poll_data(&mut self) {
        let body = json!({
            "channel": "/meta/connect",
            "clientId": self.client_id,
            "connectionType": "long-polling",
            "id":"3"
        });

        println!("{:?}", self.post_json(body));
    }
}
