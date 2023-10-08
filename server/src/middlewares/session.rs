use std::sync::Arc;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;

use crate::models::db::SharedState;

pub async fn session_middleware<B>(
    State(state): State<Arc<SharedState>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    let jar = CookieJar::from_headers(headers);
    let auth = jar.get("sambro_cookie").map(|c| c.value()).to_owned();
    println!("Cookie");
    let auth = match auth {
        Some(a) => a.to_owned(),
        None => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    println!("checking session_manager");
    let mut sm = state.session_manager.clone();
    let check_auth_store = sm.get_session(&auth).await;
    match check_auth_store {
        Some(_) => {
            println!("Login_sucess");
            let response = next.run(request).await;
            return Ok(response);
        }
        None => {
            println!("Login failure");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    // Err(StatusCode::UNAUTHORIZED)
}
