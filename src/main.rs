#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
mod schema;
mod models;
mod logtimedb;
mod shell;

use std::fs::File;
use shell::fish::Fish;

fn main() {
    let mut args = std::env::args();
    let mut dbspec = None;
    let mut shell_out = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "--fish" => {
                let path = args.next()
                    .expect("--fish requires output path");
                match File::create(path.clone()) {
                    Ok(file) => {shell_out.push(Box::new(Fish::new(file)) as Box<dyn shell::Shell>);},
                    Err(err) => {eprintln!("failed to open {}: {:?}", path, err);},
                }
            },
            "--db" => {
                let path = args.next()
                    .expect("--db requires sqlite database file location");
                dbspec = Some(path);
            },
            _ => { break; },
        }
    }
    let database = match dbspec {
        None => logtimedb::open_default().unwrap(),
        Some(path) => logtimedb::open(path).unwrap(),
    };
}
