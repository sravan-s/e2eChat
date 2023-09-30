use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

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

async fn create_user(Json(payload): Json<CreateUser>) -> Response {
    if payload.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorMessage {
                message: "Password must be at least 8 characters long".to_string(),
            }),
        )
            .into_response();
    }
    // add to database
    //
    let user = User {
        name: payload.name,
        email: payload.email,
    };
    Json(user).into_response()
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

/**
 * To do: add authentication middleware
 * To do: add database connection
 * To do Config file
 */
pub async fn bootstrap() {
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

    let app = Router::new()
        .route("/", get(|| async { "System Alive!" }))
        .route("/user", post(create_user).get(|| async { "get all users" }))
        .route("/user/:id", get(get_user).delete(delete_user))
        .route("/login", put(login_user))
        .layer(TraceLayer::new_for_http());
    tracing::debug!("listening on localhost:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
