use tokio_rusqlite::Connection;

use super::session::SessionManager;

pub struct SharedState {
    pub connection: Connection,
    pub session_manager: SessionManager,
}
