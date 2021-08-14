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
    setup();

    let mut args = env::args().skip(1);

    let url = if let Some(arg) = args.next() {
        UrlParser::parse(&arg)
    } else {
        println!("Usage: rbrowser <URL>");
        return;
    };

    match url {
        Ok(url) => {
            if let Err(e) = Browser::load(&url) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn setup() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();
}
