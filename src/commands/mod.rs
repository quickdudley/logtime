use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use crate::shell::Shell;
use crate::models;

pub fn current<A: Iterator<Item=String>, S: Shell>(args: &mut A, conn: &SqliteConnection, shell: &mut S) {
    match models::Task::current(conn) {
        None => { println!("No current task"); },
        Some(task) => { println!("{}", task.code(conn)) },
    }
}

pub fn stop<A: Iterator<Item=String>, S: Shell>(_args: &mut A, conn: &SqliteConnection, shell: &mut S) {
    models::Stretch::stop_all(conn)
        .unwrap_or_else(|e| eprintln!("{}", e));
}

pub fn start<A: Iterator<Item=String>, S: Shell>(args: &mut A, conn: &SqliteConnection, shell: &mut S) {
    match args.next() {
        Some(code) => {
            SqliteConnection::transaction(conn, || {
                models::Stretch::stop_all(conn)?;
                let (project,_task,subtask) = models::Subtask
                    ::for_code(conn, code.as_ref())?;
                subtask.begin(conn)?;
                project.directory.map(|d| shell.cd(std::path::Path::new(&d)))
                    .transpose()
                    .and_then(|cdr| cdr.and(subtask.branch)
                              .map(|branch| shell.checkout(branch.as_ref()))
                              .transpose())
                    .map_err(|e| format!("{}", e))?;
                Ok(())
            }).map(|()| ())
        },
        None => { Err(models::DbOrMiscError::from("TO-DO")) },
    }.unwrap_or_else(|e| eprintln!("{}", e));
}
