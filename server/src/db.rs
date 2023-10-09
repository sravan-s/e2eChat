use rusqlite::Connection;
use std::error::Error;
use tokio::task;

use crate::config::DB_FILE;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub fn reset_db_file() -> Result<(), Box<dyn Error>> {
    let rem = std::fs::remove_file(DB_FILE);
    if rem.is_ok() {
        println!("removed old db file");
    }
    std::fs::File::create(DB_FILE)?;
    Ok(())
}

/**
 * Todo: import config
 */
pub async fn bootstrap() -> Result<Connection, Box<dyn Error>> {
    reset_db_file()?;
    let mut connection = Connection::open(DB_FILE)?;

    println!("bootstrapping SQL");
    let connection = task::spawn_blocking(move || {
        embedded::migrations::runner().run(&mut connection).unwrap();
        connection
    })
    .await?;
    println!("bootstrapped SQL");
    Ok(connection)
}

/**
 * This does nothing much, is to print the initialized data when application starts~
 * Todo: remove this function
 */
pub fn test_db(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let mut statement = connection.prepare("SELECT * FROM USERS")?;
    let users = statement
        .query_map(rusqlite::params![], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<(String, String, String)>, rusqlite::Error>>()?;
    println!("{:?}", users);

    let mut statement = connection.prepare("SELECT * FROM PASSWORDS")?;
    let users = statement
        .query_map(rusqlite::params![], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<(String, String, String)>, rusqlite::Error>>()?;
    println!("{:?}", users);

    Ok(())
}
