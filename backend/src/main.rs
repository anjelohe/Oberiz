mod auth;
mod automation;
mod backups;
mod calendar;
mod cardigann;
mod db;
mod diagnostics;
mod download_client;
mod filesystem;
mod history;
mod importer;
mod indexers;
mod library;
mod movies;
mod overseerr;
mod profiles;
mod public_api;
mod qbittorrent;
mod releases;
mod routes;
mod rss;
mod search_api;
mod series;
#[cfg(windows)]
mod service;
mod settings;
mod tmdb;
#[cfg(windows)]
mod tray;
mod tvdb;

use automation::AutomationRuntime;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use download_client::DownloadClient;
use serde::Serialize;
use sqlx::SqlitePool;
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

struct TorrentListCache {
    fetched_at: Instant,
    torrents: Vec<qbittorrent::QBittorrentTorrent>,
}

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    http: reqwest::Client,
    automation_runtime: AutomationRuntime,
    download_client: Arc<dyn DownloadClient>,
    /// qBittorrent's full torrent list can be several megabytes. All UI
    /// refreshes share this short-lived cache instead of repeatedly making the
    /// Web API serialize the same list.
    torrent_list_cache: Arc<Mutex<Option<TorrentListCache>>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    name: &'static str,
    version: &'static str,
    database: &'static str,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let database = if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok()
    {
        "ok"
    } else {
        "error"
    };

    Json(HealthResponse {
        status: "ok",
        name: "Oberiz",
        version: env!("CARGO_PKG_VERSION"),
        database,
    })
}

/// Liveness: only confirms the process itself is responding. Never depends on
/// the database or any external service — those going down should not make a
/// container orchestrator decide to restart an otherwise-healthy process.
async fn live() -> StatusCode {
    StatusCode::OK
}

/// Readiness: confirms the database is reachable (and therefore migrations
/// already ran at startup). Still deliberately excludes TMDB/qBittorrent/
/// indexers — those are configuration the user controls, not a reason to
/// take Oberiz itself out of rotation.
async fn ready(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

async fn frontend_shell() -> Response {
    let static_directory =
        std::env::var("OBERIZ_STATIC_DIR").unwrap_or_else(|_| "frontend".to_string());
    match tokio::fs::read_to_string(std::path::Path::new(&static_directory).join("index.html"))
        .await
    {
        Ok(contents) => Html(contents).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Oberiz web interface is not installed.",
        )
            .into_response(),
    }
}

async fn build_app_state() -> anyhow::Result<AppState> {
    let db = db::connect().await?;
    settings::migrate_legacy_api_key(&db).await?;
    let http = reqwest::Client::builder()
        .user_agent(concat!("Oberiz/", env!("CARGO_PKG_VERSION")))
        .build()?;
    Ok(AppState {
        db,
        http,
        automation_runtime: AutomationRuntime::default(),
        download_client: Arc::new(download_client::QBittorrentClient),
        torrent_list_cache: Arc::new(Mutex::new(None)),
    })
}

fn spawn_schedulers(state: &AppState) {
    automation::spawn_scheduler(state.clone());
    backups::spawn_scheduler(state.clone());
    rss::spawn_scheduler(state.clone());
    importer::spawn_import_scheduler(state.clone());
    indexers::spawn_health_scheduler(state.clone());
}

/// Builds the router and serves it until `shutdown` resolves. Shared by the
/// default (portable) launch path and, on Windows, the real service: both
/// run the exact same server, differing only in how they start and how they
/// find out it's time to stop.
async fn run_server(
    state: AppState,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let static_directory = std::path::PathBuf::from(
        std::env::var("OBERIZ_STATIC_DIR").unwrap_or_else(|_| "frontend".to_string()),
    );
    let assets_directory = static_directory.join("assets");

    let internal_router = Router::new()
        .route("/api/health", get(health))
        .route("/api/health/live", get(live))
        .route("/api/health/ready", get(ready))
        .merge(routes::auth::router())
        .merge(routes::settings::router())
        .merge(routes::movies::router())
        .merge(routes::series::router())
        .merge(routes::downloads::router())
        .merge(routes::indexers::router())
        .merge(routes::profiles::router())
        .merge(routes::scheduling::router())
        .merge(routes::library::router())
        .nest_service("/assets", ServeDir::new(assets_directory))
        .route_service(
            "/oberiz-logo.png",
            ServeFile::new(static_directory.join("oberiz-logo.png")),
        )
        .route_service(
            "/oberiz-logo-middle.png",
            ServeFile::new(static_directory.join("oberiz-logo-middle.png")),
        )
        .route_service(
            "/oberiz-logo-premium.png",
            ServeFile::new(static_directory.join("oberiz-logo-premium.png")),
        )
        .route_service(
            "/paypal-donate-qr.png",
            ServeFile::new(static_directory.join("paypal-donate-qr.png")),
        )
        .fallback(get(frontend_shell));

    let public_router = routes::public::router();

    // The internal API has no CORS headers of its own: the bundled frontend
    // is always same-origin, so browsers need none, and this keeps random
    // third-party web pages from reading responses cross-origin. On top of
    // that, `auth::require_auth` gates every internal route behind the
    // optional admin-password session once one has been configured; with no
    // password set it is a no-op and behavior is unchanged from before.
    let app = internal_router
        .merge(public_router)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ))
        .with_state(state);

    // On Windows, Tokio's asynchronous address resolution can stall before
    // opening a listener when Oberiz runs as a service. Bind synchronously
    // first so the operating system either gives us the port immediately or
    // returns a useful error (for example, if another process owns it).
    let std_listener = std::net::TcpListener::bind("0.0.0.0:2032")?;
    std_listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(std_listener)?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Oberiz running on http://localhost:2032"
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    Ok(())
}

fn init_logging() {
    // OBERIZ_LOG_LEVEL is a friendlier alias for RUST_LOG (e.g. "info",
    // "oberiz=debug,tower_http=warn"); RUST_LOG wins if both are set.
    if std::env::var_os("RUST_LOG").is_none()
        && let Ok(level) = std::env::var("OBERIZ_LOG_LEVEL")
    {
        unsafe { std::env::set_var("RUST_LOG", level) };
    }
    // `fmt::init()` defaults to a very restrictive filter when neither
    // variable is set, which made useful warnings invisible in `cargo run`.
    // Keep explicit RUST_LOG / OBERIZ_LOG_LEVEL settings authoritative, but
    // make the normal foreground experience observable by default.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,html5ever=error")),
        )
        .init();
}

/// Runs the default, portable launch path: the exact same server as the
/// Windows service, in the foreground of the current process, with no
/// graceful-shutdown signal (killing the process is how this has always
/// stopped) and — on Windows — the classic embedded tray icon.
async fn run_portable() -> anyhow::Result<()> {
    let state = build_app_state().await?;
    spawn_schedulers(&state);
    #[cfg(windows)]
    tray::spawn();
    run_server(state, std::future::pending()).await
}

fn main() -> anyhow::Result<()> {
    init_logging();

    #[cfg(windows)]
    {
        match std::env::args().nth(1).as_deref() {
            Some("--service") => return service::run_as_service(),
            Some("--install-service") => return service::install(),
            Some("--uninstall-service") => return service::uninstall(),
            Some("--tray") => {
                tray::run_standalone();
                return Ok(());
            }
            _ => {}
        }
    }

    tokio::runtime::Runtime::new()?.block_on(run_portable())
}
