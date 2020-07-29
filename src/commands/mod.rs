use diesel::sqlite::SqliteConnection;
use crate::shell::Shell;
use crate::models;

pub fn current<A: Iterator<Item=String>, S: Shell>(args: &mut A, conn: &SqliteConnection, shell: &mut S) {
    match models::Task::current(conn) {
        None => { println!("No current task"); },
        Some(task) => { () },
    }
}
