use std::ops::Add;

// This is to learn about session management and middleware
//
use chrono::{DateTime, Duration, Utc};
use crossbeam_skiplist::SkipMap;
use ulid::Ulid;
// in memory session management; is not good for production
#[derive(Debug, Clone)]
pub struct Session {
    id: String,
    user_id: String,
    expires: DateTime<Utc>,
}

pub struct SessionManager {
    sessions: SkipMap<String, Session>, // I wanted to try something raro ~
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            sessions: SkipMap::new(),
        }
    }

    pub async fn add_session(&mut self, user_id: String) -> String {
        let id = Ulid::new().to_string();
        let session = Session {
            id: id.clone(),
            user_id: user_id.clone(),
            // default session length is 1 hour
            expires: Utc::now().add(Duration::hours(1)),
        };
        self.sessions.insert(id.clone(), session);
        id
    }

    pub async fn get_session(&self, id: &String) -> Option<Session> {
        let v = self.sessions.get(id);
        if v.is_none() {
            return None;
        }
        Some(v.unwrap().value().clone())
    }

    pub async fn delete_session(&mut self, id: &String) {
        self.sessions.remove(id);
    }
}
