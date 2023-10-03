use rusqlite::Connection;
use std::error::Error;

mod config;
mod db;
mod handlers;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection: Connection = db::bootstrap().await?;
    db::test_db(&connection)?;
    let _ = routes::bootstrap().await;
    Ok(())
}
