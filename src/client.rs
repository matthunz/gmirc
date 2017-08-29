extern crate reqwest;
extern crate hyper;
extern crate serde_json;

use self::hyper::header::{ContentType};
use self::reqwest::Response;
use serde_json::{Value};

pub struct Client {
    client: reqwest::Client,
    client_id: String
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest::Client::new().unwrap(),
            client_id: String::new()
        }
    }

    pub fn run(&mut self) {
        self.client_id = self.get_client_id();
        println!("{}", self.client_id);
    }

    fn post(&mut self, body: Value) -> Response {
        self.client.post("https://push.groupme.com/faye")
            .unwrap()
            .header(ContentType::json())
            .body(body.to_string())
            .send()
            .unwrap()
    }

    fn get_client_id(&mut self) -> String {
        let body = json!({
            "channel": "/meta/handshake",
            "version": "1.0",
            "supportedConnectionTypes": ["long-polling"],
            "id": "1"
        });

        let mut res = self.post(body);

        let content: Value = res.json().expect("GroupMe returned invalid json");
        content[0]["clientId"].as_str().expect("Recieved invalid response").to_owned()
    }
}
