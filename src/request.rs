use std::collections::HashMap;

use crate::Url;

pub struct Request {
    pub url: Url,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            headers: HashMap::new(),
        }
    }

    pub fn build(&self) -> String {
        let mut headers = String::from("User-Agent: rbrowser\r\n");

        for (name, value) in &self.headers {
            headers += &format!("{}: {}\r\n", &name, &value);
        }

        format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n{}\r\n",
            self.url.path, self.url.host, headers
        )
    }

    pub fn header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let request = Request::new(Url::new("http://example.com").unwrap());
        let expected = "GET / HTTP/1.1\r\n\
                        Host: example.com\r\n\
                        Connection: close\r\n\
                        User-Agent: rbrowser\r\n\r\n";
        assert_eq!(expected, request.build());
    }

    #[test]
    fn build_with_custom_header() {
        let mut request = Request::new(Url::new("http://example.com").unwrap());
        request.header("Header-A", "A");
        let expected = "GET / HTTP/1.1\r\n\
                        Host: example.com\r\n\
                        Connection: close\r\n\
                        User-Agent: rbrowser\r\n\
                        Header-A: A\r\n\r\n";
        assert_eq!(expected, request.build());
    }
}
