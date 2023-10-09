use std::ops::Add;

// This is to learn about session management and middleware
//
use chrono::{DateTime, Duration, Utc};
use moka::future::Cache;
use ulid::Ulid;
// in memory session management; is not good for production
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    pub sessions: Cache<String, Session>,
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            // meh - 10k sessions
            sessions: Cache::new(10_000),
        }
    }

    pub async fn add_session(&mut self, user_id: String) -> Session {
        let id = Ulid::new().to_string();
        let session = Session {
            id: id.clone(),
            user_id: user_id.clone(),
            // default session length is 1 hour
            expires: Utc::now().add(Duration::hours(1)),
        };
        self.sessions.insert(id.clone(), session.clone()).await;
        session
    }

    pub async fn get_session(&mut self, id: &String) -> Option<Session> {
        let v = self.sessions.get(id).await;
        let session = match v {
            Some(s) => {
                // remove session if expired
                if Utc::now().gt(&s.expires) {
                    self.sessions.remove(id).await;
                    return None;
                }
                println!("All good");
                Some(s.clone())
            }
            None => None,
        };
        session
    }

    pub async fn delete_session(&mut self, id: &String) {
        self.sessions.invalidate(id).await;
    }
}
