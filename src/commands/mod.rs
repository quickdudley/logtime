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

pub fn stop<A: Iterator<Item=String>, S: Shell>(args: &mut A, conn: &SqliteConnection, shell: &mut S) {
    models::Stretch::stop_all_at(conn, args.next())
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

pub fn cd<A: Iterator<Item=String>, S: Shell>(args: &mut A, conn: &SqliteConnection, shell: &mut S) {
    match models::Project::current(conn) {
        Err(diesel::result::Error::NotFound) => println!("No current task"),
        Err(err) => eprintln!("{}", err),
        Ok(project) => {
            match project.directory {
                Some(dir) => shell.cd(std::path::Path::new(&dir))
                    .map_or_else(|err| eprintln!("{}", err), |_| ()),
                None => eprintln!("No directory set for current project")
            }
        },
    }
}

pub fn display<A: Iterator<Item=String>, S: Shell>(args: &mut A, conn: &SqliteConnection, shell: &mut S) {
    let from = match args.next() {
        None => models::today(),
        Some(formatted) => chrono::naive::NaiveDate::parse_from_str(formatted.as_ref(), "%Y-%m-%d").unwrap(),
    };
    let time_hash = models::time_since(conn, from).unwrap();
    let mut entries = time_hash.iter().collect::<Vec<_>>();
    entries.sort_by(|(c1,_),(c2,_)| c1.cmp(c2));
    for (date, entries) in entries {
        println!("{}:", date.format("%Y-%m-%d"));
        for (code, duration) in entries.iter() {
            println!("  {}: {}:{}:{}", code,
                     duration.num_hours(),
                     duration.num_minutes() % 60,
                     duration.num_seconds() & 60);
        }
    }
}

