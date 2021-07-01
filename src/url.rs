use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Scheme {
    Http,
    Https,
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scheme::Http => write!(f, "http"),
            Scheme::Https => write!(f, "https"),
        }
    }
}

#[derive(Debug)]
pub struct Url {
    pub scheme: Scheme,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl Url {
    pub fn new(url: &str) -> Result<Self, &'static str> {
        let scheme_and_rest: Vec<&str> = url.splitn(2, "://").collect();

        let scheme = match scheme_and_rest.get(0) {
            Some(&"http") => Scheme::Http,
            Some(&"https") => Scheme::Https,
            _ => return Err("Unknown scheme, must be http or https"),
        };

        let port = match scheme {
            Scheme::Http => 80,
            Scheme::Https => 443,
        };

        let mut host_and_path = scheme_and_rest[1].splitn(2, '/');

        let host = host_and_path.next().unwrap().to_owned();
        let path = match host_and_path.next() {
            Some(path) => "/".to_owned() + path,
            None => "/".to_owned(),
        };

        Ok(Self {
            scheme,
            host,
            port,
            path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_url() {
        let url = Url::new("http://example.org/path").unwrap();
        assert_eq!(Scheme::Http, url.scheme);
        assert_eq!("example.org", url.host);
        assert_eq!(80, url.port);
        assert_eq!("/path", url.path);
    }

    #[test]
    fn new_https_url() {
        let url = Url::new("https://example.org/path").unwrap();
        assert_eq!(Scheme::Https, url.scheme);
        assert_eq!("example.org", url.host);
        assert_eq!(443, url.port);
        assert_eq!("/path", url.path);
    }

    #[test]
    fn new_url_without_path() {
        let url = Url::new("http://example.org").unwrap();
        assert_eq!("/", url.path);
    }

    #[test]
    fn new_url_without_scheme() {
        let result = Url::new("example.org");
        assert!(result.is_err());
    }
}
