use rusqlite::Connection;
use std::error::Error;
use tokio::task;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

async fn bootstrap(mut connection: Connection) -> Result<Connection, Box<dyn Error>> {
    println!("bootstrapping SQL");
    let connection = task::spawn_blocking(move || {
        embedded::migrations::runner().run(&mut connection).unwrap();
        connection
    })
    .await?;
    println!("bootstrapped SQL");
    Ok(connection)
}

fn test_db(connection: &Connection) -> Result<(), Box<dyn Error>> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut connection = Connection::open_in_memory()?;
    connection = bootstrap(connection).await?;
    test_db(&connection)?;
    Ok(())
}
