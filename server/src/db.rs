use rusqlite::Connection;
use std::error::Error;
use tokio::task;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub fn reset_db_file() -> Result<(), Box<dyn Error>> {
    let rem = std::fs::remove_file("./db/__database");
    if rem.is_ok() {
        println!("removed old db file");
    }
    std::fs::File::create("./db/__database")?;
    Ok(())
}

/**
 * Todo: import config
 */
pub async fn bootstrap() -> Result<Connection, Box<dyn Error>> {
    reset_db_file()?;
    let mut connection = Connection::open("./db/__database")?;

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
