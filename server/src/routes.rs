use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, sync::Arc};
use tokio_rusqlite::Connection;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use ulid::Ulid;

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

struct SharedState {
    connection: Connection,
}

pub fn add_user(name: &String, password: &String, email: &String) -> (String, String) {
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
    (user_insert, password_insert)
}

async fn create_user(
    State(state): State<Arc<SharedState>>,
    Json(payload): Json<CreateUser>,
) -> Response {
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

    let (user_insert, password_insert) = add_user(&payload.name, &payload.password, &payload.email);
    let transaction = state
        .connection
        .call(move |conn| {
            let t = conn.transaction()?;
            t.execute(&user_insert, [])?;
            t.execute(&password_insert, [])?;
            info!("User inserted successfully");
            Ok(())
        })
        .await;
    match transaction {
        Ok(_) => {
            info!("User inserted successfully");
            let user = User {
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

async fn get_users(State(state): State<Arc<SharedState>>) -> Response {
    let transaction = state
        .connection
        .call(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM USERS")?;
            let users = stmt
                .query_map([], |row| {
                    Ok(User {
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

async fn get_user() -> &'static str {
    "get user"
}

async fn delete_user() -> &'static str {
    "update user"
}

async fn login_user() -> &'static str {
    "login user"
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

/**
 * To do: add authentication middleware
 * To do: add database connection
 * To do Config file
 */
pub async fn bootstrap() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let conn: Connection = Connection::open("./db/__database").await?;
    let shared_state: Arc<SharedState> = Arc::new(SharedState { connection: conn });

    // todo: add authentication middleware
    let auth_routes = Router::new()
        .route("/user/:id", get(get_user).delete(delete_user))
        .route("/user", get(get_users))
        .layer(TraceLayer::new_for_http());

    let open_routes = Router::new()
        .route("/", get(|| async { "System Alive!" }))
        .route("/user", post(create_user))
        .route("/login", put(login_user));

    let app = Router::new()
        .merge(auth_routes)
        .merge(open_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    let app = app.fallback(handler_404);

    info!("listening on localhost:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
