use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Hash, PartialEq)]
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

const DEFAULT_HTTP_PORT: u16 = 80;
const DEFAULT_HTTPS_PORT: u16 = 443;

#[derive(Clone, Debug, Hash, PartialEq)]
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

        let mut port = match scheme {
            Scheme::Http => DEFAULT_HTTP_PORT,
            Scheme::Https => DEFAULT_HTTPS_PORT,
        };

        let mut host_and_path = scheme_and_rest[1].splitn(2, '/');
        let mut host = host_and_path.next().unwrap().to_owned();

        if host.contains(':') {
            let mut host_and_port = host.splitn(2, ':');
            // XXX assignment to 'host' not possible here because it's borrowed, hence using a temp
            // var
            let temp_host = host_and_port.next().unwrap().to_owned();
            port = FromStr::from_str(host_and_port.next().unwrap()).unwrap();
            host = temp_host;
        }

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

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}://{}:{}{}",
            self.scheme, self.host, self.port, self.path
        )
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
        assert_eq!(DEFAULT_HTTP_PORT, url.port);
        assert_eq!("/path", url.path);
    }

    #[test]
    fn new_https_url() {
        let url = Url::new("https://example.org/path").unwrap();
        assert_eq!(Scheme::Https, url.scheme);
        assert_eq!("example.org", url.host);
        assert_eq!(DEFAULT_HTTPS_PORT, url.port);
        assert_eq!("/path", url.path);
    }

    #[test]
    fn new_url_with_custom_port() {
        let url = Url::new("http://example.org:8080/path").unwrap();
        assert_eq!("example.org", url.host);
        assert_eq!(8080, url.port);
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

    #[test]
    fn to_string_with_http() {
        let url = Url::new("http://example.org/path").unwrap();
        assert_eq!("http://example.org:80/path", url.to_string());
    }

    #[test]
    fn to_string_with_https() {
        let url = Url::new("https://example.org/path").unwrap();
        assert_eq!("https://example.org:443/path", url.to_string());
    }
}
