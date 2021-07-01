use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::url::Url;

#[derive(Debug)]
pub struct Browser {}

impl Browser {
    pub fn load(url: &Url) {
        match Self::request(url) {
            Ok((_, body)) => Self::show(&body),
            Err(e) => eprintln!("{}", e),
        }
    }

    pub fn request(url: &Url) -> io::Result<(Vec<String>, String)> {
        let url_to_connect = format!("{}:{}", url.host, url.port);

        let mut stream = TcpStream::connect(url_to_connect)?;
        write!(
            stream,
            "GET {} HTTP/1.0\r\nHost: {}\r\n\r\n",
            url.path, url.host
        )?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        let mut lines = response.lines();
        let status_line = lines.next().unwrap();

        let mut headers = Vec::new();
        loop {
            let line = lines.next().unwrap();
            if line.is_empty() {
                break;
            }
            headers.push(line.to_string());
        }

        let body: String = lines.collect();

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
