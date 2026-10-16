use super::*;

#[test]
fn related_hosts_cannot_receive_credentials_on_arbitrary_ports() {
    let base = Url::parse("https://example.org").unwrap();
    assert!(!same_site(
        &base,
        &Url::parse("https://www.example.org:8443").unwrap(),
        true
    ));
    assert!(same_site(
        &base,
        &Url::parse("https://www.example.org").unwrap(),
        false
    ));
}

#[tokio::test]
async fn authenticated_pinned_client_preserves_credentials_without_dns_lookup_or_redirects() {
    use axum::{Router, routing::get};
    let app = Router::new().route(
        "/",
        get(|headers: axum::http::HeaderMap| async move {
            assert_eq!(headers.get("x-fixture-key").unwrap(), "synthetic");
            assert!(
                headers
                    .get("cookie")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains("fixture=yes")
            );
            (
                axum::http::StatusCode::FOUND,
                [("location", "http://127.0.0.1:1/private")],
            )
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let url = Url::parse(&format!("http://fixture.invalid:{}/", address.port())).unwrap();
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    jar.add_cookie_str("fixture=yes; Path=/", &url);
    let mut headers = HeaderMap::new();
    headers.insert("x-fixture-key", "synthetic".parse().unwrap());
    let tracker = TrackerClient {
        http: Client::new(),
        jar,
        headers,
    };
    let response = pinned_tracker_client(&tracker, "fixture.invalid", address)
        .unwrap()
        .get(url)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::FOUND);
    server.abort();
}

#[test]
fn support_messages_redact_url_credentials_and_secret_query_values() {
    let result = redact_message(
        "Error https://fixture-user:fixture-password@example.org/path?apikey=fixture-key&category=movies",
    );
    for secret in ["fixture-user", "fixture-password", "fixture-key"] {
        assert!(!result.contains(secret));
    }
    assert!(result.contains("category=movies"));
    assert!(result.contains("REDACTED"));
}

#[tokio::test]
async fn login_redirect_to_private_www_alias_is_rejected_before_sending() {
    use axum::{Router, routing::get};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = Router::new()
        .route(
            "/login",
            get(move || async move {
                (
                    axum::http::StatusCode::FOUND,
                    [("location", format!("http://www.localhost:{port}/private"))],
                )
            }),
        )
        .route(
            "/private",
            get(|| async {
                panic!("Private login redirect was followed");
                #[allow(unreachable_code)]
                ""
            }),
        );
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let def = Definition {
        id: "fixture".into(),
        name: "Fixture".into(),
        base_url: format!("http://localhost:{port}"),
        yaml: Value::Null,
        config: Default::default(),
    };
    let client = TrackerClient {
        http: Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        jar: Default::default(),
        headers: HeaderMap::new(),
    };
    assert!(
        login_send(&def, &client, client.get(format!("{}/login", def.base_url)))
            .await
            .is_err()
    );
    server.abort();
}

#[test]
fn mapped_private_and_reserved_addresses_are_denied() {
    for ip in [
        "::ffff:127.0.0.1",
        "::ffff:192.168.1.1",
        "::ffff:169.254.169.254",
        "100.64.0.1",
        "198.18.0.1",
        "240.0.0.1",
        "0.1.2.3",
        "2001:db8::1",
        "64:ff9b::7f00:1",
    ] {
        assert!(!is_public_ip(ip.parse().unwrap()), "{ip}");
    }
    for ip in ["8.8.8.8", "::ffff:8.8.8.8", "2606:4700:4700::1111"] {
        assert!(is_public_ip(ip.parse().unwrap()), "{ip}");
    }
}

#[tokio::test]
async fn untrusted_loopback_redirect_is_not_followed_and_subdomain_is_not_trusted() {
    use axum::{Router, http::StatusCode, routing::get};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let hits = Arc::new(AtomicUsize::new(0));
    let app = Router::new().route(
        "/internal",
        get({
            let hits = hits.clone();
            move || {
                let hits = hits.clone();
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    "private"
                }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let private = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    // Pin a simulated public origin to our isolated HTTP fixture. The exact
    // builder used after production address validation must not auto-redirect.
    let app = Router::new().route(
        "/redirect",
        get(move || async move {
            (
                StatusCode::FOUND,
                [("location", format!("http://{private}/internal"))],
            )
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let public = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = TrackerClient {
        http: public_fetch_client("fixture.invalid", address).unwrap(),
        jar: std::sync::Arc::new(reqwest::cookie::Jar::default()),
        headers: HeaderMap::new(),
    };
    let response = client
        .get(format!(
            "http://fixture.invalid:{}/redirect",
            address.port()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FOUND);
    let def = Definition {
        id: "fixture".into(),
        name: "Fixture".into(),
        base_url: format!("http://fixture.invalid:{}", address.port()),
        yaml: Value::Null,
        config: Default::default(),
    };
    assert!(
        follow_redirect_once(&def, &client, &[], response)
            .await
            .is_err()
    );
    assert_eq!(hits.load(Ordering::SeqCst), 0);
    let def = Definition {
        base_url: "http://localhost".into(),
        ..def
    };
    assert!(
        authorize_fetch(&def, &client, "http://child.localhost/internal")
            .await
            .is_err()
    );
    public.abort();
    server.abort();
}
