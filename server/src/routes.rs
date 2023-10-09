use axum::{
    http::StatusCode,
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use std::{error::Error, sync::Arc};
use tokio_rusqlite::Connection;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::handlers::{
    auth::login_user,
    user::{create_user, delete_user, get_user, get_users},
};
use crate::{
    config::DB_FILE,
    middlewares::session::session_middleware,
    models::{db::SharedState, session::SessionManager},
};

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

/**
 * To do Config file
 */
pub async fn bootstrap() -> Result<(), Box<dyn Error>> {
    let session_manager = SessionManager::new();
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
    let shared_state: Arc<SharedState> = Arc::new(SharedState {
        connection: conn,
        session_manager,
    });

    // todo: add authentication middleware
    let auth_routes = Router::new()
        .route("/user/:id", get(get_user).delete(delete_user))
        .route("/user", get(get_users))
        .layer(from_fn_with_state(shared_state.clone(), session_middleware))
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
