extern crate actix_web;
extern crate static_filez;
use actix_web::{App, fs};

static STATIC_FILES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/static_files"));

fn main() {
    App::new()
        .handler(
            "/static",
            static_filez::load(STATIC_FILES).unrwap())
        .finish();
}
