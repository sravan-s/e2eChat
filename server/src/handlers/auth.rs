use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::{error, info};
use ulid::Ulid;

use crate::models::{
    api::ErrorMessage,
    db::SharedState,
    user::{LoginUser, User},
};

pub async fn login_user(
    State(state): State<Arc<SharedState>>,
    Json(payload): Json<LoginUser>,
) -> Response {
    let email = payload.email.clone();
    let pwd = state
        .connection
        .call(move |conn| {
            // get user - email(username) is case insensitive
            let mut fetch_user = conn
                .prepare("SELECT id, name FROM USERS WHERE email = ?1 COLLATE NOCASE LIMIT 1")
                .unwrap();

            let mut rows = fetch_user.query([&email]).unwrap();
            let row = rows.next().unwrap().unwrap();
            let user_id = row.get::<_, String>(0)?;
            let name = row.get::<_, String>(1)?;

            // get password
            let mut fetch_password = conn
                .prepare("SELECT salt, hash FROM PASSWORDS WHERE userid = ?1")
                .unwrap();
            let mut rows = fetch_password.query([&user_id]).unwrap();
            let row = rows.next().unwrap().unwrap();
            // let salt = row.get::<_, String>(0)?;
            let hash = row.get::<_, String>(1)?;
            Ok((hash, user_id, name))
        })
        .await;
    // fetched password and hash from DB
    match pwd {
        Ok(pwd) => {
            let (hash, user_id, name) = pwd;
            let parsed_hash = PasswordHash::new(&hash).unwrap();
            let res =
                Argon2::default().verify_password(&payload.password.into_bytes(), &parsed_hash);
            // password verified
            match res {
                Ok(_) => {
                    info!("Password verified successfully");
                    return Json(User {
                        id: user_id,
                        name,
                        email: payload.email,
                    })
                    .into_response();
                }
                // password incorrect
                Err(e) => {
                    error!("Error logging in, password incorrect {}", e);
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(ErrorMessage {
                            message: "Error logging in".to_string(),
                        }),
                    )
                        .into_response();
                }
            }
        }
        Err(e) => {
            error!("Error getting password: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    message: "Error logging in".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub fn add_user(name: &String, password: &String, email: &String) -> (String, String, String) {
    let id = Ulid::new().to_string();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.clone().as_bytes(), &salt)
        .unwrap()
        .to_string();
    let user_insert = format!(
        "insert into USERS (id, name, email) values ('{}', '{}', '{}');",
        id, name, email
    );

    let password_insert = format!(
        "insert into PASSWORDS (userid, salt, hash) values ('{}', '{}', '{}');",
        id, salt, password_hash,
    );
    (user_insert, password_insert, id)
}
