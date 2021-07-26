#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

mod browser;
mod request;
mod request_handler;
mod response;
mod url;
mod url_parser;

use std::env;

use crate::browser::Browser;
use crate::url::Url;
use crate::url_parser::UrlParser;

fn main() {
    let mut args = env::args().skip(1);

    let url = if let Some(arg) = args.next() {
        UrlParser::parse(&arg)
    } else {
        println!("Usage: rbrowser <URL>");
        return;
    };

    match url {
        Ok(url) => Browser::load(&url),
        Err(e) => eprintln!("{}", e),
    }
}
