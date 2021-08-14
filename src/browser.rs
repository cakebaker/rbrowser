use gtk::gio::ApplicationFlags;
use gtk::{prelude::*, DrawingArea};
use gtk::{Application, ApplicationWindow};
use std::io;
use std::str;

use crate::request_handler::RequestHandler;
use crate::url_parser::UrlType;

#[derive(Debug)]
pub struct Browser {}

impl Browser {
    pub fn load(url_type: &UrlType) -> io::Result<()> {
        let output = match url_type {
            UrlType::Http(url) => Self::lex(&RequestHandler::request(url)?),
            UrlType::ViewSource(url) => RequestHandler::request(url)?,
            UrlType::Data {
                mediatype: _,
                base64: _,
                data,
            } => Self::lex(data),
        };

        Self::build_ui(output);
        Ok(())
    }

    fn build_ui(content: String) {
        let app = Application::new(
            Some("com.github.cakebaker.rbrowser"),
            ApplicationFlags::default(),
        );
        app.connect_activate(move |app| {
            let content = content.clone();
            let window = ApplicationWindow::builder()
                .application(app)
                .default_width(800)
                .default_height(600)
                .title("rbrowser")
                .build();

            let area = DrawingArea::new();
            #[allow(unused_must_use)]
            area.set_draw_func(move |_, ctx, _, _| {
                ctx.move_to(15.0, 15.0);
                ctx.show_text(&content);
            });
            window.set_child(Some(&area));

            window.show();
        });

        // have to pass an empty vec to disable command line parsing of Application
        app.run_with_args(&<Vec<&str>>::new());
    }

    fn lex(s: &str) -> String {
        let body = Self::get_body(s);
        let body = Self::remove_tags(body);
        Self::replace_entities(&body)
    }

    // Returns either the input string if there is no body tag, or the content between the body
    // tags. The closing body tag is optional.
    fn get_body(s: &str) -> &str {
        let mut start_pos = 0;
        let mut end_pos = s.len();

        if let Some(pos) = s.find("<body>") {
            start_pos = pos + "<body>".len();

            if let Some(pos) = s.find("</body>") {
                end_pos = pos;
            }
        }

        &s[start_pos..end_pos]
    }

    fn remove_tags(s: &str) -> String {
        let mut result = String::from("");
        let mut in_angle = false;

        for c in s.chars() {
            match c {
                '<' => in_angle = true,
                '>' => in_angle = false,
                _ if !in_angle => result.push(c),
                _ => {}
            }
        }

        result
    }

    fn replace_entities(s: &str) -> String {
        s.replace("&lt;", "<").replace("&gt;", ">")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_body() {
        let result = Browser::get_body("start<body>text</body>end");
        assert_eq!("text", result);
    }

    #[test]
    fn get_body_from_empty_string() {
        assert_eq!("", Browser::get_body(""));
    }

    #[test]
    fn get_body_from_string_without_body_tags() {
        assert_eq!("test", Browser::get_body("test"));
    }

    #[test]
    fn get_body_from_string_without_closed_body() {
        let result = Browser::get_body("start<body>text");
        assert_eq!("text", result);
    }

    #[test]
    fn remove_tags() {
        assert_eq!("test", Browser::remove_tags("<b>test</b>"));
    }

    #[test]
    fn remove_tags_from_empty_string() {
        assert_eq!("", Browser::remove_tags(""));
    }

    #[test]
    fn replace_greater_than_entities() {
        assert_eq!(">", Browser::replace_entities("&gt;"));
    }

    #[test]
    fn replace_less_than_entities() {
        assert_eq!("<", Browser::replace_entities("&lt;"));
    }

    #[test]
    fn replace_entities_in_empty_string() {
        assert_eq!("", Browser::replace_entities(""));
    }
}
