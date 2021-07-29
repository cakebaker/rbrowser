use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::SystemTime;

use crate::request::Request;
use crate::response::Response;
use crate::url::{Scheme, Url};

pub struct RequestHandler {}

impl RequestHandler {
    pub fn request(url: &Url) -> io::Result<String> {
        if let Some(cached_response) = Cache::get(url) {
            Ok(cached_response)
        } else {
            let response = RequestHandler2::request(url)?;

            if response.cache_max_age() > 0 {
                Cache::save(url, &response);
            }

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
        let url = &request.url;
        let url_to_connect = format!("{}:{}", url.host, url.port);
        let mut stream = TcpStream::connect(url_to_connect)?;
        let mut response = Vec::new();

        if url.scheme == Scheme::Https {
            let mut config = rustls::ClientConfig::new();
            config
                .root_store
                .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
            let dns_name = webpki::DNSNameRef::try_from_ascii_str(&url.host).unwrap();
            let mut session = rustls::ClientSession::new(&Arc::new(config), dns_name);

            let mut tls = rustls::Stream::new(&mut session, &mut stream);
            tls.write_all(request.build().as_bytes())?;
            let reader = tls.read_to_end(&mut response);

            // a cleanly closed TLS session leads to a ConnectionAborted error we simply ignore,
            // see https://docs.rs/rustls/0.19.1/rustls/struct.ClientSession.html#method.read
            if reader.is_err() {
                let err = reader.err().unwrap();

                if err.kind() != ErrorKind::ConnectionAborted
                    && !err.to_string().contains("CloseNotify")
                {
                    return Err(err);
                }
            }
        } else {
            write!(stream, "{}", request.build())?;

            stream.read_to_end(&mut response)?;
        }

        Ok(Response::new(&response))
    }
}

struct Cache {}

impl Cache {
    const CACHE_DIRECTORY_NAME: &'static str = "rbrowser";

    pub fn get(url: &Url) -> Option<String> {
        let hashed_url = Self::calculate_hash(&url);

        let mut dir = dirs::cache_dir().unwrap();
        dir.push(Self::CACHE_DIRECTORY_NAME);
        dir.push(hashed_url.to_string());
        dir.set_extension("txt");

        if let Ok(file_content) = fs::read_to_string(&dir) {
            if let Some((first_line, rest)) = file_content.split_once("\r\n") {
                let valid_until: u64 = first_line.parse().unwrap();

                if valid_until > Self::now() {
                    return Some(rest.to_string());
                }
            }
        }

        None
    }

    // Saves the response body in the user's cache folder, using the url's hash as the filename
    // (plus ".txt" as extension). The first line of the file describes how long the file is
    // active, in seconds since the start of the Unix Epoch (1970-01-01 00:00:00).
    pub fn save(url: &Url, response: &Response) {
        let hashed_url = Self::calculate_hash(&url);
        let valid_until = Self::now() + u64::from(response.cache_max_age());

        let file_content = valid_until.to_string() + "\r\n" + &response.body;

        let mut dir = dirs::cache_dir().unwrap();
        dir.push(Self::CACHE_DIRECTORY_NAME);
        fs::create_dir_all(&dir).expect("Unable to write cache directory");

        dir.push(hashed_url.to_string());
        dir.set_extension("txt");
        fs::write(dir, &file_content).expect("Unable to write file");
    }

    // from https://doc.rust-lang.org/std/hash/index.html
    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}