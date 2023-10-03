use tokio_rusqlite::Connection;

pub struct SharedState {
    pub connection: Connection,
}
