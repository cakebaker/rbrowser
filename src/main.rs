#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

mod browser;
mod url;

use std::env;

use crate::browser::Browser;
use crate::url::Url;

fn main() {
    let mut args = env::args();
    args.next();

    let url = match args.next() {
        Some(arg) if arg.starts_with("http://") => Url::new(&arg),
        None => {
            println!("Usage: rbrowser <URL>");
            return;
        }
        _ => {
            println!("URL must start with 'http://'");
            return;
        }
    };

    Browser::load(&url);
}
