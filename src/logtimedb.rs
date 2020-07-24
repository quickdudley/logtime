use diesel::connection::Connection;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::embed_migrations;

embed_migrations!();

pub fn open() -> Result<SqliteConnection,String> {
    let home = std::env::var("HOME")
        .map_err(|err| format!("Couldn't find home directory: {}", err))?;
    let mut storage = std::path::PathBuf::from(home);
    storage.push(".logtime.sqlite");
    let connection = SqliteConnection::establish(storage.to_string_lossy().as_ref())
        .map_err(|err| format!("Failed to open {:?}: {}", storage, err))?;
    embedded_migrations::run(&connection).map_err(|err| format!("Failed to run migrations: {}", err))?;
    Ok(connection)
}

