use rusqlite::Connection;
use std::error::Error;

mod db;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection: Connection = db::bootstrap().await?;
    db::test_db(&connection)?;
    routes::bootstrap().await;
    Ok(())
}
