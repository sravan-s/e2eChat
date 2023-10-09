use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Json, State},
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, Request, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use std::sync::Arc;
use tracing::{error, info, warn};
use ulid::Ulid;

use crate::models::{
    api::ErrorMessage,
    db::SharedState,
    user::{LoginUser, User},
};

pub async fn logout<B>(
    State(state): State<Arc<SharedState>>,
    request: Request<B>,
) -> impl IntoResponse {
    info!("trying to logout");
    let headers = request.headers();
    let jar = CookieJar::from_headers(headers);
    let auth = jar.get("sambro_cookie").map(|c| c.value()).to_owned();
    let auth = match auth {
        Some(a) => a.to_owned(),
        None => {
            warn!("Logout cookie fail");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    message: "Error logging out".to_string(),
                }),
            )
                .into_response();
        }
    };

    let mut sm = state.session_manager.clone();
    sm.delete_session(&auth).await;
    info!("Logout success");
    return (StatusCode::NO_CONTENT).into_response();
}

pub async fn login_user(
    State(state): State<Arc<SharedState>>,
    Json(payload): Json<LoginUser>,
) -> impl IntoResponse {
    info!("Trying to login");
    let email = payload.email.clone();
    let pwd = state
        .connection
        .call(move |conn| {
            info!("Selecting userId");
            // get user - email(username) is case insensitive
            let (user_id, name) = match conn.query_row(
                "SELECT id, name FROM USERS WHERE email = ?1 COLLATE NOCASE LIMIT 1",
                [&email],
                |row| {
                    let user_id = row.get::<_, String>(0)?;
                    let name = row.get::<_, String>(1)?;
                    Ok((user_id, name))
                },
            ) {
                Ok((user_id, name)) => {
                    info!("User exist");
                    (user_id, name)
                }
                Err(e) => {
                    warn!("User {} doesnt exist", e);
                    return Err(e);
                }
            };

            let hash = match conn.query_row(
                "SELECT hash FROM PASSWORDS WHERE userid = ?1",
                [&user_id],
                |row| {
                    let hash = row.get::<_, String>(0)?;
                    Ok(hash)
                },
            ) {
                Ok(hash) => {
                    info!("Hash found for user {}", &user_id);
                    hash
                }
                Err(e) => {
                    warn!("Hash not found for user {}", &user_id);
                    return Err(e);
                }
            };

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
                    let mut session_manager = state.session_manager.clone();
                    let session = session_manager.add_session(user_id.clone()).await;

                    let mut headers = HeaderMap::new();
                    let my_cookie = format!(
                        "sambro_cookie={}; Expires={}",
                        session.id,
                        session.expires.to_rfc2822()
                    );
                    let hv = HeaderValue::from_str(&my_cookie.to_string()).unwrap();
                    headers.insert(SET_COOKIE, hv);

                    // successfully logged in;
                    info!("Password verified successfully");
                    return (
                        headers,
                        Json(User {
                            id: user_id,
                            name,
                            email: payload.email,
                        }),
                    )
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
