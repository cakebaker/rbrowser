# rbrowser

Following [Web Browser Engineering](http://browser.engineering/), using Rust instead of Python to implement it.

Currently chapter 1, including all of its exercises, is implemented. Hence, rbrowser has the following features:

* HTTPS (using [rustls](https://github.com/ctz/rustls))
* redirects
* content encoding using gzip
* chunked transfer encoding
* caching
