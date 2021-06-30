#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

use std::env;
use std::io;
use std::io::{Read, Write};
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

    let result = Browser::request(url);
    println!("{:?}", result);
}

#[derive(Debug)]
struct Browser {}

impl Browser {
    pub fn request(url: Url) -> io::Result<(Vec<String>, Vec<String>)> {
        let mut body = Vec::new();
        let mut headers = Vec::new();

        if let Ok(mut stream) = TcpStream::connect(url.host.clone() + ":80") {
            write!(
                stream,
                "GET {} HTTP/1.0\r\nHost: {}\r\n\r\n",
                url.path, url.host
            )?;
            let mut response = String::new();
            stream.read_to_string(&mut response)?;

            let mut lines = response.lines();
            let status_line = lines.next().unwrap();

            loop {
                let line = lines.next().unwrap();
                if line.is_empty() {
                    break;
                }
                headers.push(line.to_string());
            }

            for line in lines {
                body.push(line.to_string());
            }
        }
        Ok((headers, body))
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
