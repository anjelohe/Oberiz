use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use serde_json::Value;
use tokio::time::{Duration, sleep};

use crate::{
    AppState,
    automation::{self, RssRelease},
    history, settings,
};

/// Call whenever something that can change whether a *previously skipped*
/// RSS release would now match — a series becoming monitored, its quality
/// profile changing, or a profile being enabled — so `sync_once` below knows
/// a skip recorded before this moment might no longer be the right call. A
/// skip stemming from the release's own content (rejected by quality rules)
/// isn't affected by any of this and stays final either way.
pub(crate) async fn mark_config_changed(db: &sqlx::SqlitePool) {
    let _ = sqlx::query(
        r#"
        INSERT INTO settings(key,value,updated_at) VALUES('rss.config_changed_at',datetime('now'),CURRENT_TIMESTAMP)
        ON CONFLICT(key) DO UPDATE SET value=datetime('now'),updated_at=CURRENT_TIMESTAMP
    "#,
    )
    .execute(db)
    .await;
}

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
                tracing::info!(
                    feeds = s.feeds,
                    new_items = s.new_items,
                    matched = s.matched,
                    grabbed = s.grabbed,
                    errors = s.errors,
                    "rss cycle completed"
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
    let mut feeds = sqlx::query_as::<_, (String, String)>(
        "SELECT indexer_id,config_json FROM indexer_configs WHERE enabled=1",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    // Higher-priority indexers are processed first so that when the same release
    // shows up in more than one feed during a cycle, the preferred indexer's copy
    // is the one that gets grabbed — `process_rss_release`'s already-grabbed check
    // then skips the same target when a lower-priority feed reaches it afterwards.
    feeds.sort_by(|(left_id, left_config), (right_id, right_config)| {
        indexer_priority(left_config)
            .cmp(&indexer_priority(right_config))
            .then_with(|| left_id.cmp(right_id))
    });
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
                // Already seen this release. A durable decision (grabbed,
                // rejected by the profile, or skipped for a reason tied to
                // that attempt) must stay final — retrying it every cycle
                // would just re-reject or re-skip the same item forever.
                // But 'error' or a still-'new' row means it never reached a
                // terminal decision (transient HTTP/DB failure, or a crash/
                // cancellation between the INSERT above and the UPDATE
                // below): that one is worth retrying on the next cycle
                // instead of silently never trying it again.
                let row: Option<(String, String)> = sqlx::query_as(
                    "SELECT outcome,processed_at FROM rss_processed_releases WHERE indexer_id=? AND release_guid=?",
                )
                .bind(&indexer_id)
                .bind(&item.guid)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);
                let retryable = match row {
                    Some((outcome, _)) if outcome == "error" || outcome == "new" => true,
                    // A 'skipped' outcome can be either content-independent
                    // (not monitored, no profile, profile disabled — all
                    // things that can change) or something that'll never
                    // change on its own ("equivalent job already active" —
                    // though reconsidering that one too is harmless, it'll
                    // just resolve correctly once the job isn't active
                    // anymore). Only reconsider it if something tracked by
                    // mark_config_changed happened after this item was last
                    // decided — otherwise every skip gets re-evaluated every
                    // single cycle forever, which is the same unbounded cost
                    // this was meant to avoid for permanent 'rejected' ones.
                    Some((outcome, processed_at)) if outcome == "skipped" => {
                        let config_changed_at: Option<String> = sqlx::query_scalar(
                            "SELECT value FROM settings WHERE key='rss.config_changed_at'",
                        )
                        .fetch_optional(&state.db)
                        .await
                        .unwrap_or(None);
                        config_changed_at.is_some_and(|changed_at| changed_at > processed_at)
                    }
                    _ => false,
                };
                if !retryable {
                    continue;
                }
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

/// Same convention as `search_api.rs`: lower number = higher priority, unset = 100.
fn indexer_priority(config: &str) -> i64 {
    serde_json::from_str::<Value>(config)
        .ok()
        .and_then(|value| value.get("oberiz_priority").cloned())
        .and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_str().and_then(|value| value.parse::<i64>().ok()))
        })
        .unwrap_or(100)
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
                .or_else(|| atom_link(block))
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
/// An Atom entry can carry several `<link>` elements — `rel="alternate"` for
/// the human-readable details page, `rel="enclosure"` for the actual
/// download. A plain first-`<link>`-wins scan picks whichever happens to be
/// written first in the feed, and a details page ahead of the enclosure
/// there meant the HTML page itself got treated as the downloadable file.
/// Prefers `rel="enclosure"` outright; otherwise the first link that isn't
/// explicitly `rel="alternate"`; only falls back to the very first link
/// found when nothing else qualifies, for a feed that doesn't use `rel` at
/// all (ordinary RSS `<link>`, or a non-compliant Atom feed).
fn atom_link(block: &str) -> Option<String> {
    let link_tag = regex::Regex::new(r"(?is)<link\b([^>]*)/?>").ok()?;
    let href = regex::Regex::new(r#"(?is)\b(?:href)=["']([^"']+)["']"#).ok()?;
    let mut fallback: Option<String> = None;
    let mut non_alternate: Option<String> = None;
    for caps in link_tag.captures_iter(block) {
        let attrs = caps.get(1)?.as_str();
        let Some(url) = href
            .captures(attrs)
            .and_then(|c| c.get(1))
            .map(|m| html_escape::decode_html_entities(m.as_str()).to_string())
        else {
            continue;
        };
        if fallback.is_none() {
            fallback = Some(url.clone());
        }
        let is_enclosure = attrs.contains("rel=\"enclosure\"") || attrs.contains("rel='enclosure'");
        if is_enclosure {
            return Some(url);
        }
        let is_alternate = attrs.contains("rel=\"alternate\"") || attrs.contains("rel='alternate'");
        if !is_alternate && non_alternate.is_none() {
            non_alternate = Some(url);
        }
    }
    non_alternate.or(fallback)
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

    #[test]
    fn prefers_the_atom_enclosure_link_over_an_earlier_alternate_link() {
        let rows = parse_feed(
            r#"<feed><entry><id>c</id><title>Show.S01E02</title>
                <link rel="alternate" type="text/html" href="https://tracker.example/details/9"/>
                <link rel="enclosure" type="application/x-bittorrent" href="https://tracker.example/dl/9.torrent"/>
            </entry></feed>"#,
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].download_url.as_deref(),
            Some("https://tracker.example/dl/9.torrent")
        );
    }

    #[test]
    fn falls_back_to_a_non_alternate_link_when_no_enclosure_exists() {
        let rows = parse_feed(
            r#"<feed><entry><id>d</id><title>Show.S01E03</title>
                <link rel="alternate" type="text/html" href="https://tracker.example/details/10"/>
                <link rel="self" href="https://tracker.example/feed"/>
            </entry></feed>"#,
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].download_url.as_deref(),
            Some("https://tracker.example/feed")
        );
    }
}
