use axum::{
    Router,
    routing::{get, post},
};

use crate::{AppState, auth};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/status", get(auth::status))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/password", post(auth::set_password))
}
