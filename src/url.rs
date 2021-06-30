#[derive(Debug)]
pub struct Url {
    pub host: String,
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

        Self { host, path }
    }
}
