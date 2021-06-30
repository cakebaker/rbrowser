#[derive(Debug)]
pub struct Url {
    pub scheme: String, // TODO probably should be an enum
    pub host: String,
    pub port: String, // TODO probably should be an int
    pub path: String,
}

impl Url {
    pub fn new(url: &str) -> Self {
        let url = &url["http://".len()..];
        let mut elements = url.splitn(2, |e| e == '/');

        let host = elements.next().unwrap().to_owned();
        let path = match elements.next() {
            Some(path) => "/".to_owned() + path,
            None => "/".to_owned(),
        };

        Self {
            scheme: "http".to_string(),
            host,
            port: "80".to_string(),
            path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_url() {
        let url = Url::new("http://example.org/path");
        assert_eq!("http", url.scheme);
        assert_eq!("example.org", url.host);
        assert_eq!("80", url.port);
        assert_eq!("/path", url.path);
    }

    #[test]
    fn new_url_without_path() {
        let url = Url::new("http://example.org");
        assert_eq!("/", url.path);
    }
}
