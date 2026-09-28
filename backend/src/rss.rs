use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use serde_json::Value;
use tokio::time::{Duration, sleep};

use crate::{
    AppState,
    automation::{self, RssRelease},
    history, settings,
};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RssIndexerState {
    pub indexer_id: String,
    pub last_synced_at: Option<String>,
    pub last_status: String,
    pub last_message: Option<String>,
    pub last_new_items: i64,
    pub last_matched_items: i64,
    pub last_grabbed_items: i64,
    pub last_error: Option<String>,
    pub updated_at: String,
}
#[derive(Debug, Serialize)]
pub struct RssStatus {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub configured_feeds: i64,
    pub states: Vec<RssIndexerState>,
}
#[derive(Debug, Serialize, Default)]
pub struct RssRunSummary {
    pub feeds: usize,
    pub new_items: usize,
    pub matched: usize,
    pub grabbed: usize,
    pub skipped: usize,
    pub errors: usize,
}

pub async fn status(
    State(state): State<AppState>,
) -> Result<Json<RssStatus>, (StatusCode, String)> {
    let enabled = setting_bool(&state, "rss.enabled", false).await?;
    let interval_minutes = setting_u64(&state, "rss.interval_minutes", 15).await?;
    let configured_feeds=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM indexer_configs WHERE enabled=1 AND json_extract(config_json,'$.rss_url') IS NOT NULL AND json_extract(config_json,'$.rss_url')<>''")
        .fetch_one(&state.db).await.map_err(internal)?;
    let states=sqlx::query_as::<_,RssIndexerState>("SELECT indexer_id,last_synced_at,last_status,last_message,last_new_items,last_matched_items,last_grabbed_items,last_error,updated_at FROM rss_sync_state ORDER BY COALESCE(last_synced_at,'') DESC")
        .fetch_all(&state.db).await.map_err(internal)?;
    Ok(Json(RssStatus {
        enabled,
        interval_minutes,
        configured_feeds,
        states,
    }))
}
pub async fn run_now(
    State(state): State<AppState>,
) -> Result<Json<RssRunSummary>, (StatusCode, String)> {
    Ok(Json(run_cycle(&state).await))
}

pub fn spawn_scheduler(state: AppState) {
    tokio::spawn(async move {
        loop {
            if setting_bool(&state, "rss.enabled", false)
                .await
                .unwrap_or(false)
            {
                let s = run_cycle(&state).await;
                println!(
                    "[RSS] feeds={} new={} matched={} grabbed={} errors={}",
                    s.feeds, s.new_items, s.matched, s.grabbed, s.errors
                );
            }
            let interval = setting_u64(&state, "rss.interval_minutes", 15)
                .await
                .unwrap_or(15)
                .clamp(5, 1440);
            sleep(Duration::from_secs(interval * 60)).await;
        }
    });
}

pub(crate) async fn run_cycle(state: &AppState) -> RssRunSummary {
    let mut summary = RssRunSummary::default();
    let feeds = sqlx::query_as::<_, (String, String)>(
        "SELECT indexer_id,config_json FROM indexer_configs WHERE enabled=1",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    for (indexer_id, config) in feeds {
        let rss_url = serde_json::from_str::<Value>(&config).ok().and_then(|v| {
            v.get("rss_url")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_string)
        });
        let Some(rss_url) = rss_url else { continue };
        summary.feeds += 1;
        let mut new_items = 0usize;
        let mut matched = 0usize;
        let mut grabbed = 0usize;
        let mut errors = 0usize;
        let response = state
            .http
            .get(&rss_url)
            .timeout(Duration::from_secs(25))
            .send()
            .await;
        let text = match response {
            Ok(r) if r.status().is_success() => match r.text().await {
                Ok(v) => v,
                Err(e) => {
                    persist(state, &indexer_id, "error", 0, 0, 0, Some(&e.to_string())).await;
                    summary.errors += 1;
                    continue;
                }
            },
            Ok(r) => {
                persist(
                    state,
                    &indexer_id,
                    "error",
                    0,
                    0,
                    0,
                    Some(&format!("HTTP {}", r.status())),
                )
                .await;
                summary.errors += 1;
                continue;
            }
            Err(e) => {
                persist(state, &indexer_id, "error", 0, 0, 0, Some(&e.to_string())).await;
                summary.errors += 1;
                continue;
            }
        };
        for mut item in parse_feed(&text) {
            item.indexer_id = indexer_id.clone();
            item.indexer_name = indexer_id.clone();
            let inserted=sqlx::query("INSERT INTO rss_processed_releases(indexer_id,release_guid,title,download_url,published_at,outcome) VALUES(?,?,?,?,?,'new') ON CONFLICT(indexer_id,release_guid) DO NOTHING")
                .bind(&indexer_id).bind(&item.guid).bind(&item.title).bind(&item.download_url).bind(&item.published).execute(&state.db).await.map(|r|r.rows_affected()>0).unwrap_or(false);
            if !inserted {
                continue;
            }
            new_items += 1;
            match automation::process_rss_release(state, &item).await {
                Ok(outcome) => {
                    if outcome.starts_with("grabbed:") {
                        grabbed += 1;
                        matched += 1
                    } else if !outcome.starts_with("ignored:") && !outcome.starts_with("skipped:") {
                        matched += 1
                    };
                    let _=sqlx::query("UPDATE rss_processed_releases SET outcome=?,detail=? WHERE indexer_id=? AND release_guid=?").bind(if outcome.starts_with("grabbed:"){"grabbed"}else if outcome.starts_with("rejected:"){"rejected"}else{"skipped"}).bind(&outcome).bind(&indexer_id).bind(&item.guid).execute(&state.db).await;
                }
                Err(e) => {
                    errors += 1;
                    let _=sqlx::query("UPDATE rss_processed_releases SET outcome='error',detail=? WHERE indexer_id=? AND release_guid=?").bind(&e).bind(&indexer_id).bind(&item.guid).execute(&state.db).await;
                }
            }
        }
        summary.new_items += new_items;
        summary.matched += matched;
        summary.grabbed += grabbed;
        summary.skipped += new_items.saturating_sub(matched);
        summary.errors += errors;
        persist(
            state,
            &indexer_id,
            if errors > 0 { "warning" } else { "ok" },
            new_items,
            matched,
            grabbed,
            None,
        )
        .await;
        history::record(
            &state.db,
            "rss.synced",
            &indexer_id,
            Some(&format!(
                "{} new · {} matched · {} grabbed",
                new_items, matched, grabbed
            )),
            if errors > 0 { "error" } else { "info" },
        )
        .await;
    }
    summary
}

fn parse_feed(body: &str) -> Vec<RssRelease> {
    let entry = regex::Regex::new(r"(?is)<(?:item|entry)\b[^>]*>(.*?)</(?:item|entry)>").unwrap();
    entry
        .captures_iter(body)
        .take(100)
        .filter_map(|caps| {
            let block = caps.get(1)?.as_str();
            let title = tag(block, "title")?;
            let guid = tag(block, "guid")
                .or_else(|| tag(block, "id"))
                .unwrap_or_else(|| title.clone());
            let link = tag(block, "enclosure")
                .or_else(|| tag(block, "link"))
                .or_else(|| tag(block, "magnet"));
            Some(RssRelease {
                indexer_id: String::new(),
                indexer_name: String::new(),
                guid,
                title,
                download_url: link.clone(),
                details_url: link,
                published: tag(block, "pubDate").or_else(|| tag(block, "published")),
            })
        })
        .collect()
}
fn tag(block: &str, name: &str) -> Option<String> {
    let attr = regex::Regex::new(&format!(
        r#"(?is)<{name}\b[^>]*\b(?:url|href)=[\"']([^\"']+)[\"'][^>]*/?>"#
    ))
    .ok()?;
    if let Some(c) = attr.captures(block) {
        return Some(html_escape::decode_html_entities(c.get(1)?.as_str()).to_string());
    }
    let node = regex::Regex::new(&format!(r"(?is)<{name}\b[^>]*>(.*?)</{name}>")).ok()?;
    let raw = node
        .captures(block)?
        .get(1)?
        .as_str()
        .trim()
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>");
    Some(html_escape::decode_html_entities(raw).to_string())
}
async fn persist(
    state: &AppState,
    id: &str,
    status: &str,
    new_items: usize,
    matched: usize,
    grabbed: usize,
    error: Option<&str>,
) {
    let _=sqlx::query("INSERT INTO rss_sync_state(indexer_id,last_synced_at,last_status,last_message,last_new_items,last_matched_items,last_grabbed_items,last_error,updated_at) VALUES(?,CURRENT_TIMESTAMP,?,NULL,?,?,?, ?,CURRENT_TIMESTAMP) ON CONFLICT(indexer_id) DO UPDATE SET last_synced_at=CURRENT_TIMESTAMP,last_status=excluded.last_status,last_new_items=excluded.last_new_items,last_matched_items=excluded.last_matched_items,last_grabbed_items=excluded.last_grabbed_items,last_error=excluded.last_error,updated_at=CURRENT_TIMESTAMP").bind(id).bind(status).bind(new_items as i64).bind(matched as i64).bind(grabbed as i64).bind(error).execute(&state.db).await;
}
async fn setting_bool(
    state: &AppState,
    key: &str,
    default: bool,
) -> Result<bool, (StatusCode, String)> {
    Ok(settings::get_value(&state.db, key)
        .await
        .map_err(internal)?
        .map(|v| v == "true")
        .unwrap_or(default))
}
async fn setting_u64(
    state: &AppState,
    key: &str,
    default: u64,
) -> Result<u64, (StatusCode, String)> {
    Ok(settings::get_value(&state.db, key)
        .await
        .map_err(internal)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(default))
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_feed;
    #[test]
    fn parses_rss_and_atom_links() {
        let rows = parse_feed(
            "<rss><item><guid>a</guid><title>Film.1080p</title><enclosure url=\"magnet:?a\"/></item><entry><id>b</id><title>Show S01E01</title><link href=\"https://x/y\"/></entry></rss>",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].download_url.as_deref(), Some("magnet:?a"));
        assert_eq!(rows[1].guid, "b");
    }
}
