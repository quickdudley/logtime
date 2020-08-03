#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
mod schema;
mod models;
mod logtimedb;
mod shell;
mod commands;

use std::fs::File;
use shell::fish::Fish;
use shell::zsh::Zsh;

fn main() {
    let mut args = std::env::args();
    let mut dbspec = None;
    let mut shell_out = Vec::new();
    let mut cmd = None;
    args.next();
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
            "--zsh" => {
                let path = args.next()
                    .expect("--zsh requires output path");
                match File::create(path.clone()) {
                    Ok(file) => {shell_out.push(Box::new(Zsh::new(file)) as Box<dyn shell::Shell>);},
                    Err(err) => {eprintln!("failed to open {}: {:?}", path, err);},
                }
            },
            "--db" => {
                let path = args.next()
                    .expect("--db requires sqlite database file location");
                dbspec = Some(path);
            },
            arg => {
                cmd = Some(arg.to_owned());
                break;
            },
        }
    }
    match cmd {
        Some(cmd) => {
            let database = match dbspec {
                None => logtimedb::open_default().unwrap(),
                Some(path) => logtimedb::open(path).unwrap(),
            };
            run_cmd(cmd.as_ref(), &mut args, &database, &mut shell_out);
        },
        None => {
            eprintln!("Usage: logtime [--fish <filename>] [--db <filename>] <command> <command args>");
        }
    }
}

fn run_cmd<A: Iterator<Item=String>, S: shell::Shell>(cmd: &str, args: &mut A, db: &diesel::sqlite::SqliteConnection, shell: &mut S) {
    match cmd {
        "current" => commands::current(args, db, shell),
        "start" => commands::start(args, db, shell),
        "stop" => commands::stop(args, db, shell),
        "cd" => commands::cd(args, db, shell),
        _ => { eprintln!("Unrecognised command!"); },
    }
}

