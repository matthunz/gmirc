use std::net::TcpListener;
use std::thread;

mod client;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6667").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut client = client::Client::new(stream.unwrap());
            client.run();
        });
    }
}
