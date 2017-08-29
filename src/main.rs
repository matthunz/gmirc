use std::net::TcpListener;
use std::thread;
use connection::Connection;

mod connection;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6667").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut c = Connection::new(stream.unwrap());
            c.run();
        });
    }
}
