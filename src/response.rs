use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::str;

#[derive(Debug, PartialEq)]
pub enum HttpStatus {
    Ok,                // 200
    MovedPermanently,  // 301
    Found,             // 302
    TemporaryRedirect, // 307
    PermanentRedirect, // 308
    NotFound,          // 404
    Unsupported,       // catch all for states not supported by this browser
}

#[derive(Debug)]
pub struct Response {
    pub status: HttpStatus,
    headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn new(bytes: &[u8]) -> Self {
        let (header, body) = match Self::find_subsequence(bytes, b"\r\n\r\n") {
            Some(pos) => (&bytes[0..pos], &bytes[(pos + b"\r\n\r\n".len())..]),
            _ => (bytes, &bytes[bytes.len()..]),
        };

        // headers should be ASCII, hence there should be no problem to turn them to UTF-8
        let header_content = String::from_utf8(header.to_vec()).unwrap();
        let mut lines = header_content.lines();

        let status = match lines.next() {
            Some(line) => Self::parse_status(line),
            _ => HttpStatus::Unsupported,
        };

        let mut headers = HashMap::new();
        for line in lines {
            if let Some((k, v)) = Self::split_into_key_value(line) {
                headers.insert(k, v);
            }
        }

        let body = if headers.contains_key("content-encoding") {
            let mut decoder = GzDecoder::new(body);
            let mut s = String::new();
            decoder.read_to_string(&mut s).unwrap();
            s
        } else {
            String::from_utf8(body.to_vec()).unwrap()
        };

        Self {
            status,
            headers,
            body,
        }
    }

    pub fn header(&self, k: &str) -> Option<&String> {
        self.headers.get(&k.to_ascii_lowercase())
    }

    pub fn is_redirect(&self) -> bool {
        [
            HttpStatus::MovedPermanently,
            HttpStatus::Found,
            HttpStatus::TemporaryRedirect,
            HttpStatus::PermanentRedirect,
        ]
        .contains(&self.status)
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    fn parse_status(s: &str) -> HttpStatus {
        s.split(' ')
            .nth(1)
            .map_or(HttpStatus::Unsupported, |status_code| match status_code {
                "200" => HttpStatus::Ok,
                "301" => HttpStatus::MovedPermanently,
                "302" => HttpStatus::Found,
                "307" => HttpStatus::TemporaryRedirect,
                "308" => HttpStatus::PermanentRedirect,
                "404" => HttpStatus::NotFound,
                _ => HttpStatus::Unsupported,
            })
    }

    fn split_into_key_value(s: &str) -> Option<(String, String)> {
        s.split_once(':')
            .map(|(k, v)| (k.to_ascii_lowercase(), v.trim().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn new() {
        let response = Response::new(
            b"HTTP/1.1 200 OK\r\n\
Server: Apache\r\n\
Content-Type: text/html\r\n\
\r\n\
Some Content",
        );
        assert_eq!(HttpStatus::Ok, response.status);
        assert_eq!("Apache".to_string(), *response.header("Server").unwrap());
        assert_eq!(
            "text/html".to_string(),
            *response.header("Content-Type").unwrap()
        );
        assert_eq!("Some Content".to_string(), response.body);
    }

    #[test]
    fn new_gzip_encoded_response() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"Hello World").unwrap();
        let mut content = encoder.finish().unwrap();
        let mut response = b"HTTP/1.1 200 OK\r\n\
            Content-Encoding: gzip\r\n\
            \r\n"
            .to_vec();
        response.append(&mut content);

        let response = Response::new(&response);
        assert_eq!(HttpStatus::Ok, response.status);
        assert_eq!("Hello World".to_string(), response.body);
    }

    #[test]
    fn parse_supported_states() {
        assert_eq!(HttpStatus::Ok, Response::parse_status("HTTP/1.1 200 OK"));
        assert_eq!(
            HttpStatus::MovedPermanently,
            Response::parse_status("HTTP/1.1 301 Moved Permanently")
        );
        assert_eq!(
            HttpStatus::Found,
            Response::parse_status("HTTP/1.1 302 Found")
        );
        assert_eq!(
            HttpStatus::TemporaryRedirect,
            Response::parse_status("HTTP/1.1 307 Temporary Redirect")
        );
        assert_eq!(
            HttpStatus::PermanentRedirect,
            Response::parse_status("HTTP/1.1 308 Permanent Redirect")
        );
        assert_eq!(
            HttpStatus::NotFound,
            Response::parse_status("HTTP/1.1 404 Not Found")
        );
    }

    #[test]
    fn parse_unsupported_status() {
        assert_eq!(
            HttpStatus::Unsupported,
            Response::parse_status("HTTP/1.1 208 Already Reported")
        );
    }

    #[test]
    fn parse_invalid_status() {
        assert_eq!(HttpStatus::Unsupported, Response::parse_status("200"));
    }

    #[test]
    fn split_into_key_value() {
        let result = Response::split_into_key_value("Header: value").unwrap();
        assert_eq!(("header".to_string(), "value".to_string()), result);
    }

    #[test]
    fn split_into_key_value_with_invalid_str() {
        assert_eq!(None, Response::split_into_key_value("header"));
    }
}
