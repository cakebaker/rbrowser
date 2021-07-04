use openssl::ssl::{SslConnector, SslMethod};
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::url::Scheme;
use crate::url::Url;

#[derive(Debug)]
pub struct Browser {}

impl Browser {
    pub fn load(url: &Url) {
        let url_to_connect = format!("{}:{}", url.host, url.port);
        let stream = TcpStream::connect(url_to_connect).unwrap();

        let result = if url.scheme == Scheme::Https {
            let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
            let stream = connector.connect(&url.host, stream).unwrap();
            Self::request(url, stream)
        } else {
            Self::request(url, stream)
        };

        match result {
            Ok((_, body)) => Self::show(&body),
            Err(e) => eprintln!("{}", e),
        }
    }

    pub fn request<T: Read + Write>(url: &Url, mut stream: T) -> io::Result<(Vec<String>, String)> {
        write!(
            stream,
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
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
