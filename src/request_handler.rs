use openssl::ssl::{SslConnector, SslMethod};
use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;

use crate::request::Request;
use crate::response::Response;
use crate::url::Scheme;
use crate::url::Url;

pub struct RequestHandler {}

impl RequestHandler {
    pub fn request(url: &Url) -> io::Result<String> {
        if Cache::contains(url) {
            Ok(Cache::get(url))
        } else {
            let response = RequestHandler2::request(url)?;
            Ok(response.body)
        }
    }
}

struct RequestHandler2 {}

impl RequestHandler2 {
    const MAX_REDIRECTS: u8 = 5;

    pub fn request(url: &Url) -> io::Result<Response> {
        let mut redirect_count = 0;
        let mut url = url;
        let mut temp_url;

        loop {
            let mut request = Request::new(url.clone());
            request.header("Accept-Encoding", "gzip");

            let response = Self::do_request(&request)?;

            if response.is_redirect() && redirect_count < Self::MAX_REDIRECTS {
                let location = response.header("Location").unwrap();
                temp_url = if location.starts_with('/') {
                    Url::new(&format!(
                        "{}://{}:{}{}",
                        url.scheme, url.host, url.port, location
                    ))
                    .unwrap()
                } else {
                    Url::new(location).unwrap()
                };
                url = &temp_url;
                redirect_count += 1;
            } else {
                break Ok(response);
            }
        }
    }

    fn do_request(request: &Request) -> io::Result<Response> {
        fn make_request<T: Read + Write>(request: &Request, mut stream: T) -> io::Result<Response> {
            write!(stream, "{}", request.build())?;

            let mut response = Vec::new();
            stream.read_to_end(&mut response)?;

            Ok(Response::new(&response))
        }

        let url = &request.url;
        let url_to_connect = format!("{}:{}", url.host, url.port);
        let stream = TcpStream::connect(url_to_connect)?;

        if url.scheme == Scheme::Https {
            let connector = SslConnector::builder(SslMethod::tls())?.build();
            if let Ok(stream) = connector.connect(&url.host, stream) {
                make_request(request, stream)
            } else {
                Err(Error::new(ErrorKind::Other, "SSL handshake failed."))
            }
        } else {
            make_request(request, stream)
        }
    }
}

struct Cache {}

impl Cache {
    // TODO implement
    pub fn contains(_url: &Url) -> bool {
        false
    }

    // TODO implement
    pub fn get(_url: &Url) -> String {
        String::from("")
    }
}
