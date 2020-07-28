use diesel::connection::Connection;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::embed_migrations;

embed_migrations!();

pub fn open_default() -> Result<SqliteConnection,String> {
    let home = std::env::var("HOME")
        .map_err(|err| format!("Couldn't find home directory: {}", err))?;
    let mut storage = std::path::PathBuf::from(home);
    storage.push(".logtime.sqlite");
    open(storage.as_path().to_string_lossy())
}

pub fn open<P: AsRef<str>>(path: P) -> Result<SqliteConnection,String> {
    let connection = SqliteConnection::establish(path.as_ref())
        .map_err(|err| format!("Failed to open {:?}: {}", path.as_ref(), err))?;
    embedded_migrations::run(&connection).map_err(|err| format!("Failed to run migrations: {}", err))?;
    Ok(connection)
}
