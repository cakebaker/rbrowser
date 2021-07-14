use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum HttpStatus {
    Ok,          // 200
    NotFound,    // 404
    Unsupported, // catch all for states not supported by this browser
}

#[derive(Debug)]
pub struct Response {
    pub status: HttpStatus,
    headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn new(s: &str) -> Self {
        let mut lines = s.lines();

        let status = match lines.next() {
            Some(line) => Self::parse_status(line),
            _ => HttpStatus::Unsupported,
        };

        let mut headers = HashMap::new();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                break;
            }

            if let Some((k, v)) = Self::split_into_key_value(line) {
                headers.insert(k, v);
            }
        }

        let body: String = lines.collect();

        Self {
            status,
            headers,
            body,
        }
    }

    pub fn header(&self, k: &str) -> Option<&String> {
        self.headers.get(&k.to_ascii_lowercase())
    }

    fn parse_status(s: &str) -> HttpStatus {
        s.split(' ')
            .nth(1)
            .map_or(HttpStatus::Unsupported, |status_code| match status_code {
                "200" => HttpStatus::Ok,
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

    #[test]
    fn new() {
        let response = Response::new(
            r#"HTTP/1.1 200 OK
Server: Apache
Content-Type: text/html

Some Content"#,
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
    fn parse_supported_states() {
        assert_eq!(HttpStatus::Ok, Response::parse_status("HTTP/1.1 200 OK"));
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
