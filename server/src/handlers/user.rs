use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{error, info};

use crate::models::{db::SharedState, user::User};
use crate::{
    handlers::auth::add_user,
    models::{api::ErrorMessage, user::CreateUser},
};

pub async fn create_user(
    State(state): State<Arc<SharedState>>,
    Json(payload): Json<CreateUser>,
) -> Response {
    // to do -> move ouside
    if payload.password.len() < 8 {
        error!("Password too short");
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorMessage {
                message: "Password must be at least 8 characters long".to_string(),
            }),
        )
            .into_response();
    }

    if payload.password.len() > 64 {
        error!("Password too long");
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorMessage {
                message: "Password must be at most 64 characters long".to_string(),
            }),
        )
            .into_response();
    }

    let (user_insert, password_insert, user_id) =
        add_user(&payload.name, &payload.password, &payload.email);
    let transaction = state
        .connection
        .call(move |conn| {
            let t = conn.transaction()?;
            t.execute(&user_insert, [])?;
            t.execute(&password_insert, [])?;
            info!("User inserted successfully");
            match t.commit() {
                Ok(_) => {
                    info!("Transaction committed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Error committing transaction: {}", e);
                    return Err(e);
                }
            }
        })
        .await;
    match transaction {
        Ok(_) => {
            println!("User inserted successfully");
            let user = User {
                id: user_id,
                name: payload.name,
                email: payload.email,
            };
            return Json(user).into_response();
        }
        Err(e) => {
            error!("Error inserting user: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    message: "Error inserting user".to_string(),
                }),
            )
                .into_response();
        }
    }
}

pub async fn get_users(State(state): State<Arc<SharedState>>) -> Response {
    let transaction = state
        .connection
        .call(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM USERS")?;
            let users = stmt
                .query_map([], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        email: row.get(2)?,
                    })
                })?
                .collect::<Result<Vec<User>, _>>()?;
            Ok(users)
        })
        .await;
    match transaction {
        Ok(users) => {
            info!("Users listed successfully");
            return Json(users).into_response();
        }
        Err(e) => {
            error!("Error reading users from Db: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    message: "Error reading users".to_string(),
                }),
            )
                .into_response();
        }
    }
}

pub async fn get_user() -> &'static str {
    "get user"
}

pub async fn delete_user() -> &'static str {
    "update user"
}
