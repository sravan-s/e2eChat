use tokio_rusqlite::Connection;

use crate::middlewares::session::SessionManager;

pub struct SharedState {
    pub connection: Connection,
    pub session_manager: SessionManager,
}
