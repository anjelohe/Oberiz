use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::{AppState, auth};

/// Overrides the app-wide 512 MiB body limit for the two routes that take
/// nothing but a password: a JSON body here is never legitimately more than
/// a few hundred bytes, so accepting anything larger just lets an
/// unauthenticated caller force many megabytes through Argon2 hashing for
/// free. The more specific `route_layer` here wins over the outer, looser
/// limit applied to the whole router in main.rs.
const CREDENTIAL_BODY_LIMIT_BYTES: usize = 4 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/status", get(auth::status))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/password", post(auth::set_password))
        .route_layer(DefaultBodyLimit::max(CREDENTIAL_BODY_LIMIT_BYTES))
}
