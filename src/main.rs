#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

use std::env;
use std::net::TcpStream;

fn main() {
    let mut args = env::args();
    args.next();

    let url = match args.next() {
        Some(arg) if arg.starts_with("http://") => Url::new(&arg),
        None => {
            println!("Usage: rbrowser <URL>");
            return;
        }
        _ => {
            println!("URL must start with 'http://'");
            return;
        }
    };

    match TcpStream::connect(url.host + ":80") {
        Ok(_) => println!("Connected"),
        Err(e) => eprintln!("{}", e),
    }
}

#[derive(Debug)]
struct Url {
    host: String,
    path: String,
}

impl Url {
    pub fn new(url: &str) -> Self {
        let url = &url["http://".len()..];
        let mut elements = url.splitn(2, |e| e == '/');

        let host = elements.next().unwrap().to_owned();
        let path = match elements.next() {
            Some(path) => "/".to_owned() + path,
            None => "/".to_owned(),
        };

        Self { host, path }
    }
}
