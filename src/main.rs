#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
mod schema;
mod models;
mod logtimedb;

fn main() {
    let database = logtimedb::open().unwrap();
}
