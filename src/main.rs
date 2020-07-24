#[macro_use] extern crate diesel_migrations;
mod logtimedb;

fn main() {
    let database = logtimedb::open().unwrap();
}
