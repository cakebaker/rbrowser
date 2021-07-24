use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
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

type HeaderMap = HashMap<String, String>;

#[derive(Debug)]
pub struct Response {
    pub status: HttpStatus,
    headers: HeaderMap,
    pub body: String,
}

impl Response {
    pub fn new(bytes: &[u8]) -> Self {
        const SEPARATOR: &[u8] = b"\r\n\r\n";

        let (header_bytes, body_bytes) = match Self::find_subsequence(bytes, SEPARATOR) {
            Some(pos) => (&bytes[0..pos], &bytes[(pos + SEPARATOR.len())..]),
            _ => (bytes, &bytes[bytes.len()..]),
        };

        let (status, headers) = HeaderParser::parse(header_bytes);
        let body = BodyParser::parse(body_bytes, &headers);

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
}

struct HeaderParser {}

impl HeaderParser {
    pub fn parse(headers: &[u8]) -> (HttpStatus, HeaderMap) {
        // headers are ASCII, hence there should be no problem to turn them to UTF-8
        let header_content = String::from_utf8(headers.to_vec()).unwrap();
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

        (status, headers)
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

struct BodyParser {}

impl BodyParser {
    pub fn parse(body: &[u8], headers: &HeaderMap) -> String {
        let body = if headers.contains_key("transfer-encoding") {
            Self::dechunk(body)
        } else {
            body.to_vec()
        };

        let body = if headers.contains_key("content-encoding") {
            Self::unzip_and_decode(&body)
        } else if let Ok(s) = String::from_utf8(body.clone()) {
            s
        } else {
            ISO_8859_1.decode(&body, DecoderTrap::Strict).unwrap()
        };

        body
    }

    fn dechunk(body: &[u8]) -> Vec<u8> {
        const TERMINATING_CHUNK_SIZE: usize = 0;
        let mut dechunked = Vec::new();
        let mut i = 0;

        // Each chunk has the format: <chunk size in hex>\r\n<chunk data>\r\n
        loop {
            let chunk_size = {
                let mut chars = Vec::new();

                while body[i] != b'\r' {
                    chars.push(body[i] as char);
                    i += 1;
                }

                let s: String = chars.into_iter().collect();
                usize::from_str_radix(&s, 16).unwrap()
            };

            if chunk_size == TERMINATING_CHUNK_SIZE {
                break;
            }

            let data_position = i + b"\r\n".len();

            dechunked.extend_from_slice(&body[data_position..(data_position + chunk_size)]);

            i = data_position + chunk_size + b"\r\n".len();
        }

        dechunked
    }

    // XXX supports UTF-8 and ISO-8859-1, everything else crashes
    fn unzip_and_decode(body: &[u8]) -> String {
        let mut decoder = GzDecoder::new(body);
        let mut s = String::new();
        if decoder.read_to_string(&mut s).is_ok() {
            s
        } else {
            let mut decoder = GzDecoder::new(body);
            let mut v = Vec::new();
            decoder.read_to_end(&mut v).unwrap();
            ISO_8859_1.decode(&v, DecoderTrap::Strict).unwrap()
        }
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
    fn new_chunk_encoded_response() {
        let response = Response::new(
            b"HTTP/1.1 200 OK\r\n\
            Transfer-Encoding: Chunked\r\n\
            \r\n\
            4\r\n\
            Wiki\r\n\
            6\r\n\
            pedia \r\n\
            E\r\n\
            in \r\n\
            \r\n\
            chunks.\r\n\
            0\r\n\
            \r\n",
        );
        assert_eq!(HttpStatus::Ok, response.status);
        assert_eq!("Wikipedia in \r\n\r\nchunks.", response.body);
    }

    #[test]
    fn parse_headers() {
        let header_bytes = b"HTTP/1.1 200 OK\r\n\
                             Header-A: Value A\r\n\
                             Header-B: Value B";

        let (status, headers) = HeaderParser::parse(header_bytes);
        assert_eq!(HttpStatus::Ok, status);
        assert_eq!("Value A".to_string(), *headers.get("header-a").unwrap());
        assert_eq!("Value B".to_string(), *headers.get("header-b").unwrap());
    }

    #[test]
    fn parse_supported_states() {
        assert_eq!(
            HttpStatus::Ok,
            HeaderParser::parse_status("HTTP/1.1 200 OK")
        );
        assert_eq!(
            HttpStatus::MovedPermanently,
            HeaderParser::parse_status("HTTP/1.1 301 Moved Permanently")
        );
        assert_eq!(
            HttpStatus::Found,
            HeaderParser::parse_status("HTTP/1.1 302 Found")
        );
        assert_eq!(
            HttpStatus::TemporaryRedirect,
            HeaderParser::parse_status("HTTP/1.1 307 Temporary Redirect")
        );
        assert_eq!(
            HttpStatus::PermanentRedirect,
            HeaderParser::parse_status("HTTP/1.1 308 Permanent Redirect")
        );
        assert_eq!(
            HttpStatus::NotFound,
            HeaderParser::parse_status("HTTP/1.1 404 Not Found")
        );
    }

    #[test]
    fn parse_unsupported_status() {
        assert_eq!(
            HttpStatus::Unsupported,
            HeaderParser::parse_status("HTTP/1.1 208 Already Reported")
        );
    }

    #[test]
    fn parse_invalid_status() {
        assert_eq!(HttpStatus::Unsupported, HeaderParser::parse_status("200"));
    }

    #[test]
    fn split_into_key_value() {
        let result = HeaderParser::split_into_key_value("Header: value").unwrap();
        assert_eq!(("header".to_string(), "value".to_string()), result);
    }

    #[test]
    fn split_into_key_value_with_invalid_str() {
        assert_eq!(None, HeaderParser::split_into_key_value("header"));
    }
}
