//! Read-only comparison of received inventory with remote metadata.
//! Writes are restricted to independent report tables.
#[cfg(test)]
#[path = "missing_tests.rs"]
mod tests;
use crate::{AppState, settings};
#[path = "missing_metadata_cache.rs"]
mod metadata_cache;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

type Error = (StatusCode, String);
type Result<T> = std::result::Result<T, Error>;
static CREATE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
const MAX_REPORT_ITEMS: usize = 20_000;

fn err(status: StatusCode, code: &str) -> Error {
    (status, code.into())
}
fn db_error(_: sqlx::Error) -> Error {
    err(StatusCode::INTERNAL_SERVER_ERROR, "report_storage_error")
}
fn opaque_id() -> String {
    let mut bytes = [0u8; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Filters {
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmdb_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tvdb_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}
fn default_scope() -> String {
    "series".into()
}
impl Filters {
    fn validate(&self, refresh: bool) -> Result<()> {
        if ![
            "series",
            "collections",
            "series_one",
            "collection_one",
            "all",
        ]
        .contains(&self.scope.as_str())
            || self.tmdb_id.is_some_and(|id| id <= 0)
            || self.tvdb_id.is_some_and(|id| id <= 0)
            || self.tmdb_id.is_some() && self.tvdb_id.is_some()
            || (self.scope.ends_with("_one") != (self.tmdb_id.is_some() || self.tvdb_id.is_some()))
            || self.tvdb_id.is_some() && self.scope != "series_one"
            || self.page == Some(0)
            || self.job_id.as_ref().is_some_and(|s| {
                s.is_empty()
                    || s.len() > 100
                    || !s
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            })
            || refresh && (self.page.is_some() || self.job_id.is_some())
        {
            return Err(err(StatusCode::BAD_REQUEST, "invalid_filter"));
        }
        Ok(())
    }
    fn signature(&self) -> String {
        serde_json::to_string(&self.base()).expect("serializable filters")
    }
    fn base(&self) -> Self {
        Self {
            page: None,
            job_id: None,
            ..self.clone()
        }
    }
}

pub(crate) async fn authorize_external(state: &AppState) -> Result<()> {
    if settings::get_value(&state.db, "api.missing_reports_enabled")
        .await
        .map_err(db_error)?
        .as_deref()
        != Some("true")
    {
        return Err(err(
            StatusCode::FORBIDDEN,
            "missing_reports_permission_required",
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovieQuery {
    movie_tmdb_id: i64,
}
pub async fn movie_collection(
    State(state): State<AppState>,
    Query(query): Query<MovieQuery>,
) -> Result<Json<Value>> {
    if query.movie_tmdb_id <= 0 {
        return Err(err(StatusCode::BAD_REQUEST, "invalid_filter"));
    }
    let sources = Sources::configured(&state).await?;
    let movie = sources
        .metadata(&format!("/movie/{}", query.movie_tmdb_id))
        .await?;
    let id = positive(&movie["belongs_to_collection"]["id"])
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "movie_has_no_collection"))?;
    Ok(Json(
        json!({"tmdb_id":id,"title":title(&movie["belongs_to_collection"]["name"],"Collection")}),
    ))
}

pub async fn read(
    State(state): State<AppState>,
    Query(filters): Query<Filters>,
) -> Result<Json<Value>> {
    filters.validate(false)?;
    let signature = filters.signature();
    let row = if let Some(id) = &filters.job_id {
        sqlx::query("SELECT report_json FROM missing_reports WHERE id=? AND signature=?")
            .bind(id)
            .bind(signature)
            .fetch_optional(&state.db)
            .await
            .map_err(db_error)?
    } else {
        // Prefer the newest finished report: while a refresh is queued or
        // running its placeholder has no groups yet and would hide the last
        // real result. A placeholder (then an error) is only returned when no
        // finished report exists for this scope.
        sqlx::query(
            "SELECT report_json FROM missing_reports WHERE signature=? ORDER BY CASE state WHEN 'error' THEN 2 WHEN 'queued' THEN 1 WHEN 'running' THEN 1 ELSE 0 END, rowid DESC LIMIT 1",
        )
        .bind(signature)
        .fetch_optional(&state.db)
        .await
        .map_err(db_error)?
    };
    let row = row.ok_or_else(|| {
        err(
            if filters.job_id.is_some() {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::CONFLICT
            },
            "report_not_found",
        )
    })?;
    let report: Value = serde_json::from_str(&row.get::<String, _>("report_json"))
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "report_storage_error"))?;
    Ok(Json(page_report(report, filters.page.unwrap_or(1))?))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(filters): Json<Filters>,
) -> Result<(StatusCode, Json<Value>)> {
    refresh_inner(state, headers, filters, None).await
}
async fn refresh_inner(
    state: AppState,
    headers: HeaderMap,
    filters: Filters,
    sources: Option<Sources>,
) -> Result<(StatusCode, Json<Value>)> {
    filters.validate(true)?;
    let key = headers
        .get("idempotency-key")
        .and_then(|s| s.to_str().ok())
        .unwrap_or("");
    if !(16..=100).contains(&key.len())
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(err(StatusCode::BAD_REQUEST, "idempotency_key_required"));
    }
    let _guard = CREATE_LOCK.lock().await;
    // Expire retry keys after seven days. Preserve the latest valid report for
    // every scope while bounding older terminal reports to three per scope.
    sqlx::query("DELETE FROM missing_report_actions WHERE job_id IN (SELECT id FROM missing_reports WHERE state NOT IN ('queued','running') AND created_at < datetime('now','-7 days'))")
        .execute(&state.db).await.map_err(db_error)?;
    sqlx::query("DELETE FROM missing_reports WHERE state NOT IN ('queued','running') AND id NOT IN (SELECT id FROM (SELECT id, row_number() OVER (PARTITION BY signature ORDER BY (state='error'), rowid DESC) AS rank FROM missing_reports WHERE state NOT IN ('queued','running')) WHERE rank <= 3)")
        .execute(&state.db).await.map_err(db_error)?;
    let signature = filters.signature();
    if let Some(row)=sqlx::query("SELECT a.signature,r.report_json FROM missing_report_actions a JOIN missing_reports r ON r.id=a.job_id WHERE a.action_key=?")
        .bind(key).fetch_optional(&state.db).await.map_err(db_error)? {
        if row.get::<String,_>("signature")!=signature { return Err(err(StatusCode::CONFLICT,"idempotency_key_conflict")); }
        let report=serde_json::from_str(&row.get::<String,_>("report_json")).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"report_storage_error"))?;
        return Ok((StatusCode::OK,Json(page_report(report,1)?)));
    }
    if let Some(row)=sqlx::query("SELECT id,report_json FROM missing_reports WHERE signature=? AND state IN ('queued','running') ORDER BY rowid DESC LIMIT 1")
        .bind(&signature).fetch_optional(&state.db).await.map_err(db_error)? {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM missing_report_actions WHERE job_id=?").bind(row.get::<String,_>("id")).fetch_one(&state.db).await.map_err(db_error)?;
        if count >= 128 { return Err(err(StatusCode::TOO_MANY_REQUESTS,"too_many_retry_keys")); }
        sqlx::query("INSERT INTO missing_report_actions(action_key,signature,job_id) VALUES(?,?,?)").bind(key).bind(&signature).bind(row.get::<String,_>("id")).execute(&state.db).await.map_err(db_error)?;
        let report=serde_json::from_str(&row.get::<String,_>("report_json")).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"report_storage_error"))?;
        return Ok((StatusCode::ACCEPTED,Json(page_report(report,1)?)));
    }
    let running: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM missing_reports WHERE state IN ('queued','running')",
    )
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;
    if running >= 2 {
        return Err(err(StatusCode::TOO_MANY_REQUESTS, "too_many_checks"));
    }
    let sources = match sources {
        Some(value) => value,
        None => Sources::configured(&state).await?,
    };
    ensure_report_capacity(&state.db, &signature).await?;
    let id = opaque_id();
    let now: String = sqlx::query_scalar("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')")
        .fetch_one(&state.db)
        .await
        .map_err(db_error)?;
    let report = json!({"contract_version":1,"job_id":id,"report_id":id,"state":"queued","generated_at":now,
        "inventory_generated_at":null,"inventory_version":null,"filters":filters.base(),"totals":{"groups":null,"episodes":null,"movies":null},"warnings":[],"groups":[],"page":1,"next_page":null});
    let mut tx = state.db.begin().await.map_err(db_error)?;
    sqlx::query(
        "INSERT INTO missing_reports(id,signature,state,report_json) VALUES(?,?,'queued',?)",
    )
    .bind(&id)
    .bind(&signature)
    .bind(report.to_string())
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    sqlx::query("INSERT INTO missing_report_actions(action_key,signature,job_id) VALUES(?,?,?)")
        .bind(key)
        .bind(signature)
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    tx.commit().await.map_err(db_error)?;
    let response = report.clone();
    tokio::spawn(async move {
        let mut result = report;
        result["state"] = json!("running");
        if save(&state, &id, &result).await.is_err() {
            return;
        }
        let today = sqlx::query_scalar::<_, String>("SELECT date('now')")
            .fetch_one(&state.db)
            .await;
        let outcome = match today {
            Ok(today) => tokio::time::timeout(
                Duration::from_secs(1200),
                calculate(&sources, &filters, &today, &mut result),
            )
            .await
            .map_err(|_| err(StatusCode::GATEWAY_TIMEOUT, "check_timeout"))
            .and_then(|v| v),
            Err(_) => Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "report_storage_error",
            )),
        };
        if let Err((_, code)) = outcome {
            result["state"] = json!("error");
            result["groups"] = json!([]);
            result["warnings"] = json!([code]);
        }
        if let Ok(now) =
            sqlx::query_scalar::<_, String>("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')")
                .fetch_one(&state.db)
                .await
        {
            result["generated_at"] = json!(now);
        }
        let _ = save(&state, &id, &result).await;
    });
    Ok((StatusCode::ACCEPTED, Json(response)))
}
async fn save(state: &AppState, id: &str, report: &Value) -> Result<()> {
    let mut encoded = report.to_string();
    let mut state_name = report["state"].as_str().unwrap_or("error");
    if encoded.len() > 16 * 1024 * 1024 {
        let mut limited = report.clone();
        limited["groups"] = json!([]);
        limited["state"] = json!("error");
        limited["warnings"] = json!(["report_storage_limit"]);
        encoded = limited.to_string();
        state_name = "error";
    }
    sqlx::query("UPDATE missing_reports SET state=?,report_json=? WHERE id=?")
        .bind(state_name)
        .bind(encoded)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(db_error)?;
    Ok(())
}
pub(crate) async fn recover_interrupted(db: &sqlx::SqlitePool) -> Result<()> {
    for row in sqlx::query(
        "SELECT id,report_json FROM missing_reports WHERE state IN ('queued','running')",
    )
    .fetch_all(db)
    .await
    .map_err(db_error)?
    {
        let mut report: Value = serde_json::from_str(&row.get::<String, _>("report_json"))
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "report_storage_error"))?;
        report["state"] = json!("error");
        report["warnings"] = json!(["check_interrupted"]);
        sqlx::query("UPDATE missing_reports SET state='error',report_json=? WHERE id=?")
            .bind(report.to_string())
            .bind(row.get::<String, _>("id"))
            .execute(db)
            .await
            .map_err(db_error)?;
    }
    Ok(())
}

/// Fixed split boundaries make every job's pages stable; keep well below 1 MiB.
fn page_report(mut report: Value, page: usize) -> Result<Value> {
    let groups = report["groups"].as_array().cloned().unwrap_or_default();
    let mut pages: Vec<Vec<Value>> = vec![Vec::new()];
    let mut size = 0;
    for group in groups {
        let bytes = group.to_string().len();
        if bytes > 700_000 {
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "report_item_too_large",
            ));
        }
        if !pages.last().unwrap().is_empty()
            && (size + bytes > 700_000 || pages.last().unwrap().len() >= 100)
        {
            pages.push(Vec::new());
            size = 0;
        }
        size += bytes;
        pages.last_mut().unwrap().push(group);
    }
    if page > pages.len() {
        return Err(err(StatusCode::BAD_REQUEST, "invalid_page"));
    }
    report["groups"] = json!(pages[page - 1]);
    report["page"] = json!(page);
    report["next_page"] = if page < pages.len() {
        json!(page + 1)
    } else {
        Value::Null
    };
    Ok(report)
}

#[derive(Clone)]
struct Sources {
    db: sqlx::SqlitePool,
    metadata_days: u64,
    completed_days: u64,
    stale_metadata: std::sync::Arc<std::sync::atomic::AtomicBool>,
    client: reqwest::Client,
    inventory: Inventory,
    tmdb_base: String,
    tmdb_key: String,
}
impl Sources {
    async fn configured(state: &AppState) -> Result<Self> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT received_at,items_json FROM received_library_inventory WHERE id=1",
        )
        .fetch_optional(&state.db)
        .await
        .map_err(db_error)?;
        let (received, encoded) =
            row.ok_or_else(|| err(StatusCode::CONFLICT, "cinetta_inventory_not_received"))?;
        let saved: BTreeMap<String, Vec<Value>> = serde_json::from_str(&encoded)
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "invalid_inventory"))?;
        let inventory = Inventory {
            items: saved.into_values().flatten().collect(),
            generated: json!(received),
            version: Value::Null,
            warnings: BTreeSet::new(),
            complete: false,
        };
        let tmdb_key = settings::get_value(&state.db, "tmdb.api_key")
            .await
            .map_err(db_error)?
            .unwrap_or_default();
        if tmdb_key.trim().is_empty() {
            return Err(err(
                StatusCode::PRECONDITION_REQUIRED,
                "tmdb_not_configured",
            ));
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "http_client_error"))?;
        Ok(Self {
            db: state.db.clone(),
            metadata_days: settings::get_value(&state.db, "missing.metadata_days")
                .await
                .map_err(db_error)?
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(7)
                .clamp(1, 365),
            completed_days: settings::get_value(&state.db, "missing.completed_days")
                .await
                .map_err(db_error)?
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(90)
                .clamp(7, 365),
            stale_metadata: Default::default(),
            client,
            inventory,
            tmdb_base: "https://api.themoviedb.org/3".into(),
            tmdb_key,
        })
    }
    async fn fetch_metadata(&self, path: &str) -> Result<Value> {
        // TMDB answers 429 when briefly over its rate limit; waiting a little
        // (as it asks) keeps a large check from turning into a partial one.
        for attempt in 1..=3u64 {
            let request = self
                .client
                .get(format!("{}{path}", self.tmdb_base))
                .query(&[("language", "es-ES")]);
            let response = crate::tmdb::apply_tmdb_auth(request, &self.tmdb_key)
                .send()
                .await
                .map_err(|_| err(StatusCode::BAD_GATEWAY, "metadata_unavailable"))?;
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt < 3 {
                let wait = response
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.trim().parse::<u64>().ok())
                    .unwrap_or(2 * attempt)
                    .clamp(1, 10);
                tokio::time::sleep(Duration::from_secs(wait)).await;
                continue;
            }
            return bounded_json(response, 4 * 1024 * 1024, "metadata_unavailable").await;
        }
        Err(err(StatusCode::BAD_GATEWAY, "metadata_unavailable"))
    }
}
async fn bounded_json(mut response: reqwest::Response, limit: usize, code: &str) -> Result<Value> {
    if !response.status().is_success() {
        return Err(err(StatusCode::BAD_GATEWAY, code));
    }
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err(err(StatusCode::BAD_GATEWAY, "response_too_large"));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| err(StatusCode::BAD_GATEWAY, code))?
    {
        if bytes.len() + chunk.len() > limit {
            return Err(err(StatusCode::BAD_GATEWAY, "response_too_large"));
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| err(StatusCode::BAD_GATEWAY, "invalid_provider_response"))
}

#[derive(Clone)]
struct Inventory {
    items: Vec<Value>,
    generated: Value,
    version: Value,
    warnings: BTreeSet<String>,
    complete: bool,
}
fn positive(value: &Value) -> Option<i64> {
    value.as_i64().filter(|n| *n > 0)
}
fn title(value: &Value, fallback: &str) -> String {
    value
        .as_str()
        .unwrap_or(fallback)
        .chars()
        .take(240)
        .collect()
}
fn released(value: &Value, today: &str) -> bool {
    let Some(date) = value.as_str() else {
        return false;
    };
    if date.len() != 10 {
        return false;
    }
    let parts: Vec<_> = date.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    year >= 1800 && day >= 1 && day <= days && date <= today
}

async fn calculate(s: &Sources, f: &Filters, today: &str, report: &mut Value) -> Result<()> {
    let inv = s.inventory.clone();
    report["inventory_generated_at"] = inv.generated;
    report["inventory_version"] = inv.version;
    let mut warnings = inv.warnings;
    if !inv.complete {
        warnings.insert("inventory_incomplete".into());
    }
    let mut movies = BTreeSet::new();
    let mut episodes = BTreeMap::<i64, BTreeSet<(i64, i64)>>::new();
    let mut expanded_episodes = 0usize;
    for item in inv.items {
        match item["type"].as_str() {
            Some("movie") => {
                if let Some(id) = positive(&item["tmdb_id"]) {
                    movies.insert(id);
                }
            }
            Some("episode") => {
                if let (Some(id), Some(season), Some(episode)) = (
                    positive(&item["series_tmdb_id"]),
                    item["season_number"].as_i64(),
                    item["episode_number"].as_i64(),
                ) && season >= 0
                    && episode > 0
                {
                    // A multi-episode file (S01E01E02) may report its last
                    // episode; otherwise its later episodes would be listed
                    // as missing although they are on disk.
                    let last = bounded_episode_end(episode, item["episode_number_end"].as_i64());
                    let known = episodes.entry(id).or_default();
                    for number in episode..=last {
                        if known.insert((season, number)) {
                            expanded_episodes += 1;
                            if expanded_episodes > 500_000 {
                                return Err(err(StatusCode::BAD_GATEWAY, "inventory_too_large"));
                            }
                        }
                    }
                }
            }
            _ => {
                warnings.insert("unidentified_items".into());
            }
        }
    }
    let mut groups = Vec::new();
    let mut group_count = 0usize;
    let mut episode_count = 0usize;
    let mut movie_count = 0usize;
    let mut series_ids = BTreeSet::new();
    if ["series", "all"].contains(&f.scope.as_str()) {
        series_ids.extend(episodes.keys().copied());
    }
    if f.scope == "series_one" {
        let id = if let Some(id) = f.tmdb_id {
            Some(id)
        } else {
            let found = s
                .metadata(&format!(
                    "/find/{}?external_source=tvdb_id",
                    f.tvdb_id.unwrap()
                ))
                .await?;
            found["tv_results"]
                .as_array()
                .filter(|v| v.len() == 1)
                .and_then(|v| positive(&v[0]["id"]))
        };
        if let Some(id) = id {
            series_ids.insert(id);
        } else {
            warnings.insert("series_not_identified".into());
        }
    }
    if series_ids.len() > 1000 {
        warnings.insert("title_limit_reached".into());
    }
    // Consume each title and season before fetching the next one. Never retain
    // an entire library's metadata responses in memory.
    for id in series_ids.into_iter().take(1000) {
        let detail = s.metadata(&format!("/tv/{id}")).await;
        let detail = match detail {
            Ok(v) => v,
            Err(_) => {
                warnings.insert("metadata_unavailable".into());
                continue;
            }
        };
        let name = title(&detail["name"], "Series");
        let mut affected = false;
        let Some(seasons) = detail["seasons"].as_array() else {
            warnings.insert("metadata_incomplete".into());
            continue;
        };
        for season in seasons.iter().take(200) {
            if episode_count >= MAX_REPORT_ITEMS {
                warnings.insert("item_limit_reached".into());
                break;
            }
            let Some(number) = season["season_number"].as_i64().filter(|n| *n >= 0) else {
                continue;
            };
            let details = match s.metadata(&format!("/tv/{id}/season/{number}")).await {
                Ok(v) => v,
                Err(_) => {
                    warnings.insert("metadata_unavailable".into());
                    continue;
                }
            };
            let Some(rows) = details["episodes"].as_array() else {
                warnings.insert("metadata_incomplete".into());
                continue;
            };
            let mut missing = BTreeMap::new();
            for e in rows {
                if episode_count + missing.len() >= MAX_REPORT_ITEMS {
                    warnings.insert("item_limit_reached".into());
                    break;
                }
                let Some(ep) = positive(&e["episode_number"]) else {
                    continue;
                };
                if released(&e["air_date"], today)
                    && !episodes.get(&id).is_some_and(|v| v.contains(&(number, ep)))
                {
                    missing.insert(ep,json!({"title":title(&e["name"],"Episode"),"tmdb_id":positive(&e["id"]),"season_number":number,"episode_number":ep,"release_date":format!("{}T00:00:00Z",e["air_date"].as_str().unwrap()),"availability":"unknown"}));
                }
            }
            if !missing.is_empty() {
                affected = true;
                episode_count += missing.len();
                let rows: Vec<_> = missing.into_values().collect();
                for (part, chunk) in rows.chunks(500).enumerate() {
                    groups.push(json!({"type":"series","tmdb_id":id,"tvdb_id":f.tvdb_id,"title":format!("{name} · S{number:02} · {}",part+1),"items":chunk}));
                }
            }
        }
        if seasons.len() > 200 {
            warnings.insert("season_limit_reached".into());
        }
        if affected {
            group_count += 1;
        }
        if episode_count >= MAX_REPORT_ITEMS {
            warnings.insert("item_limit_reached".into());
            break;
        }
    }
    let mut collections = BTreeSet::new();
    if ["collections", "all"].contains(&f.scope.as_str()) {
        if movies.len() > 2000 {
            warnings.insert("title_limit_reached".into());
        }
        for id in movies.iter().take(2000) {
            let result = s.metadata(&format!("/movie/{id}")).await;
            match result {
                Ok(v) => {
                    if let Some(c) = positive(&v["belongs_to_collection"]["id"]) {
                        collections.insert(c);
                    }
                }
                Err(_) => {
                    warnings.insert("metadata_unavailable".into());
                }
            }
        }
    }
    if f.scope == "collection_one" {
        collections.insert(f.tmdb_id.unwrap());
    }
    for id in collections {
        if episode_count + movie_count >= MAX_REPORT_ITEMS {
            warnings.insert("item_limit_reached".into());
            break;
        }
        let details = match s.metadata(&format!("/collection/{id}")).await {
            Ok(v) => v,
            Err(_) => {
                warnings.insert("metadata_unavailable".into());
                continue;
            }
        };
        let Some(parts) = details["parts"].as_array() else {
            warnings.insert("metadata_incomplete".into());
            continue;
        };
        let mut missing = BTreeMap::new();
        for movie in parts {
            if episode_count + movie_count + missing.len() >= MAX_REPORT_ITEMS {
                warnings.insert("item_limit_reached".into());
                break;
            }
            if let Some(mid) = positive(&movie["id"])
                && !movies.contains(&mid)
                && released(&movie["release_date"], today)
            {
                missing.insert((movie["release_date"].as_str().unwrap().to_string(),mid),json!({"title":title(&movie["title"],"Movie"),"tmdb_id":mid,"season_number":null,"episode_number":null,"release_date":format!("{}T00:00:00Z",movie["release_date"].as_str().unwrap()),"availability":"unknown"}));
            }
        }
        if !missing.is_empty() {
            group_count += 1;
            movie_count += missing.len();
            let rows: Vec<_> = missing.into_values().collect();
            for (part, chunk) in rows.chunks(500).enumerate() {
                groups.push(json!({"type":"collection","tmdb_id":id,"tvdb_id":null,"title":format!("{} · {}",title(&details["name"],"Collection"),part+1),"items":chunk}));
            }
        }
    }
    if s.stale_metadata.load(std::sync::atomic::Ordering::Relaxed) {
        warnings.insert("metadata_stale".into());
    }
    report["state"] = json!(if !warnings.is_empty() {
        "partial"
    } else if groups.is_empty() {
        "no_matches"
    } else {
        "complete"
    });
    report["totals"] = json!({"groups":group_count,"episodes":episode_count,"movies":movie_count});
    report["groups"] = json!(groups);
    report["warnings"] = json!(warnings);
    Ok(())
}

fn bounded_episode_end(first: i64, end: Option<i64>) -> i64 {
    end.filter(|end| *end >= first && first.checked_add(50).is_some_and(|max| *end <= max))
        .unwrap_or(first)
}

async fn ensure_report_capacity(db: &sqlx::SqlitePool, signature: &str) -> Result<()> {
    // Reports are a disposable cache. When storage is full, drop the oldest
    // finished reports instead of refusing every new check from then on
    // (each single series or collection is its own scope, so a fixed cap
    // would otherwise be reached after enough different checks).
    loop {
        let (count, bytes): (i64, i64) = sqlx::query_as(
            "SELECT count(*),COALESCE(sum(length(report_json)),0) FROM missing_reports",
        )
        .fetch_one(db)
        .await
        .map_err(db_error)?;
        if count < 200 && bytes < 64 * 1024 * 1024 {
            break;
        }
        let evicted = sqlx::query(
            "DELETE FROM missing_reports WHERE id=(SELECT id FROM missing_reports WHERE state NOT IN ('queued','running') AND id != COALESCE((SELECT id FROM missing_reports WHERE signature=? AND state NOT IN ('queued','running','error') ORDER BY rowid DESC LIMIT 1),'') ORDER BY rowid ASC LIMIT 1)",
        )
        .bind(signature)
        .execute(db)
        .await
        .map_err(db_error)?
        .rows_affected();
        if evicted == 0 {
            return Err(err(
                StatusCode::TOO_MANY_REQUESTS,
                "report_storage_capacity",
            ));
        }
    }
    Ok(())
}
