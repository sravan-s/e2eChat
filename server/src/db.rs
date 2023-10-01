use rusqlite::Connection;
use std::error::Error;
use tokio::task;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

/**
 * Todo: import config
 */
pub async fn bootstrap() -> Result<Connection, Box<dyn Error>> {
    let mut connection = Connection::open("./db/v1")?;

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
