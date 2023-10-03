use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use std::{error::Error, sync::Arc};
use tokio_rusqlite::Connection;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::DB_FILE,
    models::{db::SharedState, user::User},
};
use crate::{
    handlers::auth::{add_user, login_user},
    models::{api::ErrorMessage, user::CreateUser},
};

async fn create_user(
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

async fn get_users(State(state): State<Arc<SharedState>>) -> Response {
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

async fn get_user() -> &'static str {
    "get user"
}

async fn delete_user() -> &'static str {
    "update user"
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
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let conn: Connection = Connection::open(DB_FILE).await?;
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
