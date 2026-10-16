use super::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[test]
fn episode_ranges_are_bounded_without_integer_overflow() {
    assert_eq!(bounded_episode_end(i64::MAX, Some(i64::MAX)), i64::MAX);
    assert_eq!(bounded_episode_end(1, Some(i64::MAX)), 1);
    assert_eq!(bounded_episode_end(10, Some(9)), 10);
    assert_eq!(bounded_episode_end(1, Some(51)), 51);
    assert_eq!(bounded_episode_end(1, Some(52)), 1);
    assert_eq!(bounded_episode_end(1, None), 1);
}

async fn state() -> AppState {
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    AppState {
        db,
        http: reqwest::Client::new(),
        automation_runtime: Default::default(),
        download_client: Arc::new(crate::download_client::QBittorrentClient),
        torrent_list_cache: Arc::new(Mutex::new(None)),
        grab_lock: Arc::new(Mutex::new(())),
    }
}
fn filters(scope: &str, id: Option<i64>) -> Filters {
    Filters {
        scope: scope.into(),
        tmdb_id: id,
        tvdb_id: None,
        page: None,
        job_id: None,
    }
}
fn report(f: &Filters, id: &str) -> Value {
    json!({"contract_version":1,"job_id":id,"report_id":id,"filters":f,"state":"queued","groups":[],"warnings":[],"generated_at":"2026-10-16T00:00:00Z","totals":{"groups":null,"episodes":null,"movies":null}})
}

#[tokio::test]
async fn terminal_reports_are_retained_with_quota_and_previous_valid_report() {
    let state = state().await;
    let f = filters("series", None);
    for n in 0..8 {
        let mut value = report(&f, &format!("old{n}"));
        value["state"] = json!(if n == 7 { "error" } else { "partial" });
        sqlx::query("INSERT INTO missing_reports(id,signature,state,report_json) VALUES(?,?,?,?)")
            .bind(format!("old{n}"))
            .bind(f.signature())
            .bind(value["state"].as_str())
            .bind(value.to_string())
            .execute(&state.db)
            .await
            .unwrap();
    }
    let mut headers = HeaderMap::new();
    headers.insert("idempotency-key", "quota-fixture-0001".parse().unwrap());
    // No configured sources: housekeeping must not initiate network work.
    assert!(
        refresh_inner(state.clone(), headers.clone(), f.clone(), None)
            .await
            .is_err()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM missing_reports")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count, 3);
    let latest = read(State(state.clone()), Query(f)).await.unwrap().0;
    assert_eq!(latest["job_id"], "old6");
    for n in 0..200 {
        let f = filters("series_one", Some(n + 1));
        let mut value = report(&f, &format!("scope{n}"));
        value["state"] = json!("partial");
        sqlx::query(
            "INSERT INTO missing_reports(id,signature,state,report_json) VALUES(?,?,'partial',?)",
        )
        .bind(format!("scope{n}"))
        .bind(f.signature())
        .bind(value.to_string())
        .execute(&state.db)
        .await
        .unwrap();
    }
    // A full store no longer blocks checks forever: the oldest finished
    // reports are evicted and the request proceeds (here it then stops only
    // because no sources are configured).
    let error = refresh_inner(state.clone(), headers, filters("all", None), None)
        .await
        .unwrap_err();
    assert_eq!(error.1, "cinetta_not_configured");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM missing_reports")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count, 203, "Invalid configuration must not evict reports");
    ensure_report_capacity(&state.db, &filters("series", None).signature())
        .await
        .unwrap();
    assert_eq!(
        read(State(state.clone()), Query(filters("series", None)))
            .await
            .unwrap()
            .0["job_id"],
        "old6"
    );
    let newest_kept: i64 =
        sqlx::query_scalar("SELECT count(*) FROM missing_reports WHERE id='scope199'")
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(newest_kept, 1);
    let oldest_kept: i64 =
        sqlx::query_scalar("SELECT count(*) FROM missing_reports WHERE id='old4'")
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(oldest_kept, 0);
}

#[tokio::test]
async fn latest_report_prefers_a_finished_one_over_a_running_placeholder() {
    let state = state().await;
    let f = filters("series", None);
    for (id, state_name) in [("done", "complete"), ("pending", "running")] {
        let mut value = report(&f, id);
        value["state"] = json!(state_name);
        sqlx::query("INSERT INTO missing_reports(id,signature,state,report_json) VALUES(?,?,?,?)")
            .bind(id)
            .bind(f.signature())
            .bind(state_name)
            .bind(value.to_string())
            .execute(&state.db)
            .await
            .unwrap();
    }
    let latest = read(State(state.clone()), Query(f)).await.unwrap().0;
    assert_eq!(latest["job_id"], "done");
}

#[test]
fn strict_filters_dates_and_urls() {
    for f in [
        filters("bad", None),
        filters("series_one", None),
        filters("series", Some(1)),
        filters("collection_one", Some(-1)),
    ] {
        assert!(f.validate(true).is_err());
    }
    let mut f = filters("series_one", Some(1));
    f.page = Some(2);
    assert!(f.validate(true).is_err());
    assert!(f.validate(false).is_ok());
    assert!(released(&json!("2024-02-29"), "2026-10-16"));
    for date in [
        Value::Null,
        json!(""),
        json!("2099-01-01"),
        json!("2023-02-29"),
        json!("2026-13-01"),
        json!("2026-10-160"),
    ] {
        assert!(!released(&date, "2026-10-16"));
    }
    assert!(validate_base("http://localhost:3000/cinetta").is_ok());
    for url in [
        "file:///private",
        "https://user:password@server",
        "http://server/?key=secret",
        "http://server/#key",
    ] {
        assert!(validate_base(url).is_err());
    }
}

#[test]
fn report_pagination_is_stable_and_bounded() {
    let mut report = report(&filters("series", None), "fixed_job");
    report["groups"] = json!(
        (0..205)
            .map(|id| json!({"tmdb_id":id,"title":"Series","items":[]}))
            .collect::<Vec<_>>()
    );
    let first = page_report(report.clone(), 1).unwrap();
    let second = page_report(report.clone(), 2).unwrap();
    assert_eq!(first["groups"].as_array().unwrap().len(), 100);
    assert_eq!(second["groups"][0]["tmdb_id"], 100);
    assert_eq!(first["job_id"], second["job_id"]);
    assert_eq!(first["next_page"], 2);
    assert_eq!(
        page_report(report.clone(), 3).unwrap()["groups"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert!(page_report(report, 4).is_err());
    assert!(second.to_string().len() < 1024 * 1024);
}

async fn providers() -> (
    Sources,
    tokio::task::JoinHandle<()>,
    Arc<Mutex<Vec<String>>>,
) {
    use axum::{
        extract::{Path, Query},
        routing::get,
    };
    let calls = Arc::new(Mutex::new(Vec::new()));
    let app=axum::Router::new().route("/inventory",get({let calls=calls.clone();move|headers:HeaderMap,Query(query):Query<BTreeMap<String,String>>|{let calls=calls.clone();async move {
        assert_eq!(headers["authorization"],"Bearer synthetic-inventory-key");calls.lock().await.push("inventory".into());
        let second=query.contains_key("cursor");
        Json(json!({"contract_version":1,"snapshot_id":"fixed_snapshot","inventory_version":"fixed_version","generated_at":"2026-10-16T00:00:00Z","complete":false,"warnings":["physical_availability_unknown"],"items":if second{json!([{"type":"movie","tmdb_id":10,"availability":"unknown"}])}else{json!([{"type":"episode","series_tmdb_id":20,"season_number":1,"episode_number":1,"availability":"unknown"},{"type":"episode","series_tmdb_id":20,"season_number":1,"episode_number":2,"episode_number_end":3,"availability":"offline"}])},"next_cursor":if second{Value::Null}else{json!("page_two")}}))
    }}})).route("/3/{*path}",get({let calls=calls.clone();move|Path(path):Path<String>|{let calls=calls.clone();async move{
        calls.lock().await.push(path.clone());
        Json(match path.as_str(){
            "movie/10"=>json!({"belongs_to_collection":{"id":30}}),
            "collection/30"=>json!({"name":"Collection","parts":[{"id":10,"title":"Existing","release_date":"2000-01-01"},{"id":11,"title":"Missing","release_date":"2020-01-01"},{"id":12,"title":"Future","release_date":"2099-01-01"},{"id":13,"title":"Unknown","release_date":null}]}),
            "tv/20"=>json!({"name":"Series","seasons":[{"season_number":0},{"season_number":1}]}),
            "tv/20/season/0"=>json!({"episodes":[{"id":9999,"episode_number":1,"name":"Special","air_date":"2020-01-01"}]}),
            "tv/20/season/1"=>json!({"episodes":(1..=603).map(|ep|json!({"id":ep,"episode_number":ep,"name":format!("Episode {ep}"),"air_date":if ep==603{Value::Null}else if ep==602{json!("2099-01-01")}else{json!("2020-01-01")}})).collect::<Vec<_>>()}),
            "find/123"=>json!({"tv_results":[{"id":20}]}),
            _=>json!({}),
        })
    }}}));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (
        Sources {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            inventory_url: format!("http://{address}/inventory"),
            inventory_key: "synthetic-inventory-key".into(),
            tmdb_base: format!("http://{address}/3"),
            tmdb_key: "synthetic-metadata-key".into(),
        },
        server,
        calls,
    )
}

#[tokio::test]
async fn real_http_comparison_excludes_future_unknown_and_offline_indexed_items_splits_large_series()
 {
    let (sources, server, calls) = providers().await;
    let f = filters("all", None);
    let mut result = report(&f, "job_demo");
    calculate(&sources, &f, "2026-10-16", &mut result)
        .await
        .unwrap();
    assert_eq!(result["state"], "partial");
    assert_eq!(
        result["totals"],
        json!({"groups":2,"episodes":599,"movies":1})
    );
    assert_eq!(result["inventory_version"], "fixed_version");
    assert_eq!(result["groups"].as_array().unwrap().len(), 4);
    assert!(
        result["groups"]
            .as_array()
            .unwrap()
            .iter()
            .all(|g| g["items"].as_array().unwrap().len() <= 500)
    );
    assert!(!result.to_string().contains("Future"));
    assert!(!result.to_string().contains("Existing"));
    assert!(
        result["warnings"]
            .as_array()
            .unwrap()
            .contains(&json!("inventory_incomplete"))
    );
    assert_eq!(
        calls
            .lock()
            .await
            .iter()
            .filter(|v| *v == "inventory")
            .count(),
        2
    );
    let mut missing = filters("series_one", None);
    missing.tvdb_id = Some(123);
    let mut by_tvdb = report(&missing, "job_tvdb");
    calculate(&sources, &missing, "2026-10-16", &mut by_tvdb)
        .await
        .unwrap();
    assert_eq!(by_tvdb["totals"]["episodes"], 599);
    server.abort();
}

#[tokio::test]
async fn jobs_are_durable_idempotent_read_only_and_preserve_previous_report() {
    let state = state().await;
    let (sources, server, _) = providers().await;
    let f = filters("all", None);
    let mut headers = HeaderMap::new();
    headers.insert("idempotency-key", "synthetic-action-0001".parse().unwrap());
    let (_, Json(queued)) = refresh_inner(state.clone(), headers.clone(), f.clone(), Some(sources))
        .await
        .unwrap();
    let id = queued["job_id"].as_str().unwrap().to_owned();
    let (_, Json(repeated)) = refresh_inner(state.clone(), headers.clone(), f.clone(), None)
        .await
        .unwrap();
    assert_eq!(queued["job_id"], repeated["job_id"]);
    assert_eq!(
        refresh_inner(state.clone(), headers, filters("series", None), None)
            .await
            .unwrap_err()
            .0,
        StatusCode::CONFLICT
    );
    for _ in 0..200 {
        let status: String = sqlx::query_scalar("SELECT state FROM missing_reports WHERE id=?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        if status != "queued" && status != "running" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let Json(saved) = read(State(state.clone()), Query(f.clone())).await.unwrap();
    assert_eq!(saved["state"], "partial");
    for table in [
        "movies",
        "series",
        "media_files",
        "download_jobs",
        "media_requests",
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(count, 0, "mutated {table}");
    }
    let mut newer = report(&f, "job_failed");
    newer["state"] = json!("error");
    sqlx::query(
        "INSERT INTO missing_reports(id,signature,state,report_json) VALUES(?,?,'error',?)",
    )
    .bind("job_failed")
    .bind(f.signature())
    .bind(newer.to_string())
    .execute(&state.db)
    .await
    .unwrap();
    assert_eq!(
        read(State(state.clone()), Query(f.clone()))
            .await
            .unwrap()
            .0["job_id"],
        id
    );
    let mut exact = f.clone();
    exact.job_id = Some("job_failed".into());
    assert_eq!(
        read(State(state.clone()), Query(exact)).await.unwrap().0["state"],
        "error"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM missing_reports")
            .fetch_one(&state.db)
            .await
            .unwrap(),
        2
    );
    assert!(
        read(State(state.clone()), Query(filters("series", None)))
            .await
            .is_err()
    );
    assert!(authorize_external(&state).await.is_err());
    settings::set_value(&state.db, "api.missing_reports_enabled", "true")
        .await
        .unwrap();
    authorize_external(&state).await.unwrap();
    server.abort();
}

#[tokio::test]
async fn restart_marks_interrupted_jobs_without_restarting_network_work() {
    let state = state().await;
    let f = filters("series", None);
    sqlx::query("INSERT INTO missing_reports(id,signature,state,report_json) VALUES('interrupted',?,'running',?)").bind(f.signature()).bind(report(&f,"interrupted").to_string()).execute(&state.db).await.unwrap();
    recover_interrupted(&state.db).await.unwrap();
    assert_eq!(
        read(State(state), Query(f)).await.unwrap().0["state"],
        "error"
    );
}

#[tokio::test]
async fn external_http_requires_key_and_explicit_permission_before_json_or_jobs() {
    let state = state().await;
    settings::set_value(&state.db, "api.enabled", "true")
        .await
        .unwrap();
    settings::set_value(
        &state.db,
        "api.key_hash",
        &settings::hash_api_key("synthetic-api-key"),
    )
    .await
    .unwrap();
    let app = crate::routes::public::router(state.clone()).with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::new();
    let url = format!("http://{address}/api/v1/library/missing/refresh");
    for key in ["", "wrong"] {
        let response = client
            .post(&url)
            .header("x-api-key", key)
            .header("content-type", "application/json")
            .body("invalid json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    let response = client
        .post(&url)
        .header("x-api-key", "synthetic-api-key")
        .header("content-type", "application/json")
        .body("invalid json")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    settings::set_value(&state.db, "api.missing_reports_enabled", "true")
        .await
        .unwrap();
    let response = client
        .get(url.trim_end_matches("/refresh"))
        .header("x-api-key", "synthetic-api-key")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM missing_reports")
            .fetch_one(&state.db)
            .await
            .unwrap(),
        0
    );
    server.abort();
}

#[tokio::test]
async fn inventory_credentials_are_not_returned_and_invalid_urls_do_not_partially_save() {
    let state = state().await;
    settings::set_value(&state.db, "cinetta.inventory_key", "private-synthetic-key")
        .await
        .unwrap();
    let Json(value) = settings::get_settings(State(state.clone())).await.unwrap();
    let json = serde_json::to_string(&value).unwrap();
    assert!(value.cinetta_inventory_key_set);
    assert!(!value.missing_reports_enabled);
    assert!(!json.contains("private-synthetic-key"));
    let request = settings::UpdateSettingsRequest {
        cinetta_url: Some("http://user:secret@localhost".into()),
        missing_reports_enabled: Some(true),
        ..Default::default()
    };
    assert!(
        settings::update_settings(State(state.clone()), Json(request))
            .await
            .is_err()
    );
    assert!(authorize_external(&state).await.is_err());
}
