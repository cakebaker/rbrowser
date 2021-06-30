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

    Browser::load(&url);
}

#[derive(Debug)]
struct Browser {}

impl Browser {
    pub fn load(url: &Url) {
        match Self::request(url) {
            Ok((_, body)) => Self::show(&body),
            Err(e) => eprintln!("{}", e),
        }
    }

    pub fn request(url: &Url) -> io::Result<(Vec<String>, String)> {
        let mut body = String::new();
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

            body = lines.collect();
        }
        Ok((headers, body))
    }

    pub fn show(body: &str) {
        let mut in_angle = false;

        for c in body.chars() {
            match c {
                '<' => in_angle = true,
                '>' => in_angle = false,
                _ if !in_angle => print!("{}", c),
                _ => {}
            }
        }
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
