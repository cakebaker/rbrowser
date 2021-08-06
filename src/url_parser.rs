use crate::url::UrlError;
use crate::Url;

#[derive(Debug)]
pub enum UrlType {
    Http(Url),
    ViewSource(Url),
    Data {
        mediatype: Option<String>,
        base64: bool,
        data: String,
    },
}

pub struct UrlParser {}

impl UrlParser {
    pub fn parse(url: &str) -> Result<UrlType, UrlError> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(UrlType::Http(Url::new(url)?))
        } else if let Some(stripped) = url.strip_prefix("view-source:") {
            Ok(UrlType::ViewSource(Url::new(stripped)?))
        } else {
            url.strip_prefix("data:")
                .map_or(Err(UrlError::UnknownScheme), |stripped| {
                    Self::parse_data_url(stripped)
                })
        }
    }

    // Function expects a string in the form: [<mediatype>][;base64],<data> and always returns an UrlType::Data
    // see also https://datatracker.ietf.org/doc/html/rfc2397
    fn parse_data_url(s: &str) -> Result<UrlType, UrlError> {
        if !s.contains(',') {
            return Err(UrlError::InvalidDataUrlFormat);
        }
        let mut split = s.splitn(2, ',');
        let mut base64 = false;

        let mediatype = split.next().and_then(|mediatype| {
            let mt = mediatype.strip_suffix(";base64").map_or_else(
                || Some(mediatype.to_string()),
                |mt| {
                    base64 = true;
                    Some(mt.to_string())
                },
            );

            if mt == Some("".to_string()) {
                None
            } else {
                mt
            }
        });

        let data = split.next().map_or("".to_string(), ToString::to_string);

        Ok(UrlType::Data {
            mediatype,
            base64,
            data,
        })
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
        let result = UrlParser::parse("data:text/html,test").unwrap();
        match result {
            UrlType::Data {
                mediatype,
                base64,
                data,
            } => {
                assert_eq!(Some("text/html".to_string()), mediatype);
                assert_eq!(false, base64);
                assert_eq!("test", data);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_data_url_with_base64_set() {
        let result = UrlParser::parse("data:image/gif;base64,image").unwrap();
        match result {
            UrlType::Data {
                mediatype,
                base64,
                data,
            } => {
                assert_eq!(Some("image/gif".to_string()), mediatype);
                assert_eq!(true, base64);
                assert_eq!("image", data);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_data_url_with_no_data() {
        let result = UrlParser::parse("data:,").unwrap();
        match result {
            UrlType::Data {
                mediatype,
                base64,
                data,
            } => {
                assert_eq!(None, mediatype);
                assert_eq!(false, base64);
                assert_eq!("", data);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_invalid_data_url() {
        assert!(UrlParser::parse("data:nodata").is_err());
    }

    #[test]
    fn parse_url_with_unknown_scheme() {
        assert!(UrlParser::parse("mailto:x@y.com").is_err());
    }
}
