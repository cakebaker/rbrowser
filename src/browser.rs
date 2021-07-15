use openssl::ssl::{SslConnector, SslMethod};
use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;

use crate::response::Response;
use crate::url::Scheme;
use crate::url::Url;
use crate::url_parser::UrlType;

#[derive(Debug)]
pub struct Browser {}

impl Browser {
    pub fn load(url_type: &UrlType) {
        match url_type {
            UrlType::Http(url) | UrlType::ViewSource(url) => match Self::request(url) {
                Ok(response) => match url_type {
                    UrlType::ViewSource(_) => println!("{}", response.body),
                    _ => Self::show(&response.body),
                },
                Err(e) => eprintln!("{}", e),
            },
            // XXX the "view" doesn't support mediatype and base64 encoding
            UrlType::Data {
                mediatype: _,
                base64: _,
                data,
            } => Self::show(data),
        }
    }

    fn request(url: &Url) -> io::Result<Response> {
        fn make_request<T: Read + Write>(url: &Url, mut stream: T) -> io::Result<Response> {
            write!(
                stream,
                "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: rbrowser\r\n\r\n",
                url.path, url.host
            )?;
            let mut response = String::new();
            stream.read_to_string(&mut response)?;

            Ok(Response::new(&response))
        }

        let url_to_connect = format!("{}:{}", url.host, url.port);
        let stream = TcpStream::connect(url_to_connect)?;

        if url.scheme == Scheme::Https {
            let connector = SslConnector::builder(SslMethod::tls())?.build();
            if let Ok(stream) = connector.connect(&url.host, stream) {
                make_request(url, stream)
            } else {
                Err(Error::new(ErrorKind::Other, "SSL handshake failed."))
            }
        } else {
            make_request(url, stream)
        }
    }

    fn show(s: &str) {
        let body = Self::get_body(s);
        let body = Self::remove_tags(body);
        println!("{}", Self::replace_entities(&body));
    }

    // Returns either the input string if there is no body tag, or the content between the body
    // tags. The closing body tag is optional.
    fn get_body(s: &str) -> &str {
        let mut start_pos = 0;
        let mut end_pos = s.len();

        if let Some(pos) = s.find("<body>") {
            start_pos = pos + "<body>".len();

            if let Some(pos) = s.find("</body>") {
                end_pos = pos;
            }
        }

        &s[start_pos..end_pos]
    }

    fn remove_tags(s: &str) -> String {
        let mut result = String::from("");
        let mut in_angle = false;

        for c in s.chars() {
            match c {
                '<' => in_angle = true,
                '>' => in_angle = false,
                _ if !in_angle => result.push(c),
                _ => {}
            }
        }

        result
    }

    fn replace_entities(s: &str) -> String {
        s.replace("&lt;", "<").replace("&gt;", ">")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_body() {
        let result = Browser::get_body("start<body>text</body>end");
        assert_eq!("text", result);
    }

    #[test]
    fn get_body_from_empty_string() {
        assert_eq!("", Browser::get_body(""));
    }

    #[test]
    fn get_body_from_string_without_body_tags() {
        assert_eq!("test", Browser::get_body("test"));
    }

    #[test]
    fn get_body_from_string_without_closed_body() {
        let result = Browser::get_body("start<body>text");
        assert_eq!("text", result);
    }

    #[test]
    fn remove_tags() {
        assert_eq!("test", Browser::remove_tags("<b>test</b>"));
    }

    #[test]
    fn remove_tags_from_empty_string() {
        assert_eq!("", Browser::remove_tags(""));
    }

    #[test]
    fn replace_greater_than_entities() {
        assert_eq!(">", Browser::replace_entities("&gt;"));
    }

    #[test]
    fn replace_less_than_entities() {
        assert_eq!("<", Browser::replace_entities("&lt;"));
    }

    #[test]
    fn replace_entities_in_empty_string() {
        assert_eq!("", Browser::replace_entities(""));
    }
}
