use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Scheme {
    Http,
    Https,
}

impl Scheme {
    const fn default_port(&self) -> u16 {
        match self {
            Scheme::Http => 80,
            Scheme::Https => 443,
        }
    }
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
pub enum UrlError {
    InvalidDataUrlFormat,
    InvalidPort,
    NoHost,
    UnknownScheme,
}

impl Error for UrlError {}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDataUrlFormat => write!(f, "Invalid data url: ',' missing"),
            Self::InvalidPort => write!(f, "Invalid port"),
            Self::NoHost => write!(f, "Missing host"),
            Self::UnknownScheme => write!(f, "Unknown scheme, must be http or https"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct Url {
    pub scheme: Scheme,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl Url {
    pub fn new(url: &str) -> Result<Self, UrlError> {
        let (scheme, url_without_scheme) = match url.split_once("://") {
            Some((_, "")) => return Err(UrlError::NoHost),
            Some(("http", url_without_scheme)) => (Scheme::Http, url_without_scheme),
            Some(("https", url_without_scheme)) => (Scheme::Https, url_without_scheme),
            _ => return Err(UrlError::UnknownScheme),
        };

        let (host, path) = match url_without_scheme.split_once('/') {
            Some((host, path)) => (host, "/".to_owned() + path),
            None => (url_without_scheme, "/".to_owned()),
        };

        let (host, port) = match host.split_once(':') {
            Some((host, port)) if port.parse::<u16>().is_ok() => {
                (host, port.parse::<u16>().unwrap())
            }
            None => (host, scheme.default_port()),
            _ => return Err(UrlError::InvalidPort),
        };

        let host = host.to_string();

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
    fn new_url_with_scheme_only() {
        let result = Url::new("http://");
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
