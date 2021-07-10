use crate::Url;

#[derive(Debug)]
pub enum UrlType {
    Http(Url),
    ViewSource(Url),
    Data(String),
}

pub struct UrlParser {}

impl UrlParser {
    pub fn parse(url: &str) -> Result<UrlType, &'static str> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(UrlType::Http(Url::new(url)?))
        } else if let Some(stripped) = url.strip_prefix("view-source:") {
            Ok(UrlType::ViewSource(Url::new(stripped)?))
        } else if let Some(stripped) = url.strip_prefix("data:") {
            Ok(UrlType::Data(stripped.to_string()))
        } else {
            Err("Unknown scheme")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_http_url() {
        let result = UrlParser::parse("http://example.com").unwrap();
        let expected = Url::new("http://example.com").unwrap();
        match result {
            UrlType::Http(url) => assert_eq!(expected, url),
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_https_url() {
        let result = UrlParser::parse("https://example.com").unwrap();
        let expected = Url::new("https://example.com").unwrap();
        match result {
            UrlType::Http(url) => assert_eq!(expected, url),
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_viewsource_url() {
        let result = UrlParser::parse("view-source:http://example.com").unwrap();
        let expected = Url::new("http://example.com").unwrap();
        match result {
            UrlType::ViewSource(url) => assert_eq!(expected, url),
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_data_url() {
        let result = UrlParser::parse("data:test").unwrap();
        match result {
            UrlType::Data(s) => assert_eq!("test", s),
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_url_with_unknown_scheme() {
        assert!(UrlParser::parse("mailto:x@y.com").is_err());
    }
}
