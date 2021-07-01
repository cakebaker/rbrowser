#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

mod browser;
mod url;

use std::env;

use crate::browser::Browser;
use crate::url::Url;

fn main() {
    let mut args = env::args();
    args.next();

    let url = if let Some(arg) = args.next() {
        Url::new(&arg)
    } else {
        println!("Usage: rbrowser <URL>");
        return;
    };

    match url {
        Ok(url) => Browser::load(&url),
        Err(e) => eprintln!("{}", e),
    }
}
