use std::ops::Add;

// This is to learn about session management and middleware
//
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use ulid::Ulid;
// in memory session management; is not good for production
#[derive(Debug, Clone)]
pub struct Session {
    id: String,
    user_id: String,
    expires: DateTime<Utc>,
}

pub struct SessionManager {
    sessions: DashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            sessions: DashMap::new(),
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

    pub async fn get_session(&mut self, id: &String) -> Option<Session> {
        let v = self.sessions.get(id);
        let session = match v {
            Some(s) => {
                // remove session if expired
                if Utc::now().lt(&s.expires) {
                    self.sessions.remove(id);
                    return None;
                }
                Some(s.clone())
            }
            None => None,
        };
        session
    }

    pub async fn delete_session(&mut self, id: &String) {
        self.sessions.remove(id);
    }
}
