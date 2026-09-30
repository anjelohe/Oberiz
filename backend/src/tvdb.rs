use serde::Deserialize;

use crate::{AppState, settings};

const API_BASE: &str = "https://api4.thetvdb.com/v4";

#[derive(Deserialize)]
struct LoginResponse {
    data: LoginData,
}

#[derive(Deserialize)]
struct LoginData {
    token: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    tvdb_id: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
    name: Option<String>,
    year: Option<String>,
}

/// Resolve a TVDB series ID only when the user has opted in with an API key.
/// Search failures are deliberately non-fatal: an indexer can still use title,
/// TMDB and IMDb variables when TVDB is unavailable.
pub async fn find_series_id(
    state: &AppState,
    title: &str,
    original_title: Option<&str>,
    year: Option<i32>,
) -> Option<String> {
    let api_key = settings::get_value(&state.db, "tvdb.api_key")
        .await
        .ok()
        .flatten()?;
    if api_key.trim().is_empty() {
        return None;
    }

    let login = state
        .http
        .post(format!("{API_BASE}/login"))
        .json(&serde_json::json!({ "apikey": api_key }))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<LoginResponse>()
        .await
        .ok()?;

    let query = original_title
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(title);
    let response = state
        .http
        .get(format!("{API_BASE}/search"))
        .bearer_auth(login.data.token)
        .query(&[("query", query), ("type", "series")])
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<SearchResponse>()
        .await
        .ok()?;

    let wanted = normalize(title);
    let original = original_title.map(normalize);
    response
        .data
        .into_iter()
        .filter(|candidate| {
            let name = candidate.name.as_deref().map(normalize).unwrap_or_default();
            let title_matches = name == wanted || original.as_deref() == Some(name.as_str());
            let year_matches = year.is_none()
                || candidate
                    .year
                    .as_deref()
                    .and_then(|value| value.parse::<i32>().ok())
                    .is_none_or(|value| (value - year.unwrap_or_default()).abs() <= 1);
            title_matches && year_matches
        })
        .find_map(|candidate| candidate.tvdb_id.or(candidate.id))
        .and_then(value_to_string)
}

fn value_to_string(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) if !value.trim().is_empty() => Some(value),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
