use super::*;
use axum::{
    Router,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[tokio::test]
async fn repeated_and_concurrent_calls_reuse_login_and_renew_only_on_expiry() {
    let logins = Arc::new(AtomicUsize::new(0));
    let expired = Arc::new(AtomicBool::new(false));
    let app = Router::new()
        .route(
            "/api/v2/auth/login",
            post({
                let logins = logins.clone();
                let expired = expired.clone();
                move || {
                    let logins = logins.clone();
                    let expired = expired.clone();
                    async move {
                        logins.fetch_add(1, Ordering::SeqCst);
                        expired.store(false, Ordering::SeqCst);
                        (
                            [("set-cookie", "SID=isolated-test-session; Path=/; HttpOnly")],
                            "Ok.",
                        )
                    }
                }
            }),
        )
        .route(
            "/api/v2/app/version",
            get({
                let expired = expired.clone();
                move |headers: HeaderMap| {
                    let expired = expired.clone();
                    async move {
                        if headers.get("cookie").is_some_and(|h| {
                            h.to_str()
                                .unwrap_or("")
                                .contains("SID=isolated-test-session")
                        }) && !expired.load(Ordering::SeqCst)
                        {
                            "v5.1.0".into_response()
                        } else {
                            StatusCode::FORBIDDEN.into_response()
                        }
                    }
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let config = Arc::new(QBittorrentConfig {
        host: "127.0.0.1".into(),
        port,
        username: "fixture".into(),
        password: "fixture-only".into(),
        https: false,
    });
    for _ in 0..25 {
        let response = authenticated_get(&config, "/api/v2/app/version")
            .await
            .unwrap();
        assert_eq!(response.response.text().await.unwrap(), "v5.1.0");
    }
    assert_eq!(logins.load(Ordering::SeqCst), 1);
    expired.store(true, Ordering::SeqCst);
    let mut calls = Vec::new();
    for _ in 0..12 {
        let config = config.clone();
        calls.push(tokio::spawn(async move {
            authenticated_get(&config, "/api/v2/app/version")
                .await
                .unwrap()
                .response
                .text()
                .await
                .unwrap()
        }));
    }
    for call in calls {
        assert_eq!(call.await.unwrap(), "v5.1.0");
    }
    assert_eq!(logins.load(Ordering::SeqCst), 2);
    let first = shared_session(&config).await.unwrap();
    let again = shared_session(&config).await.unwrap();
    assert!(Arc::ptr_eq(&first, &again));
    server.abort();
}
