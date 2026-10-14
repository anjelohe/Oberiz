//! Small Radarr/Sonarr v3 adapter used only by Overseerr.
use crate::{
    AppState, profiles,
    public_api::{self, CreateRequest},
    settings,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
};
use serde_json::{Value, json};

/// Decodes a query parameter properly instead of splitting the raw query
/// string and matching a literal prefix against it: a standard URL encoder
/// turns `:` into `%3A` (Overseerr's own HTTP client does this), so
/// `term=tmdb%3A603` never matched a hand-rolled `strip_prefix("term=tmdb:")`
/// even though it's the same value tmdb:603 means once decoded.
fn query_param(uri: &Uri, key: &str) -> Option<String> {
    let query = uri.query()?;
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    uri: &Uri,
) -> Result<(), (StatusCode, String)> {
    let enabled = settings::get_value(&state.db, "overseerr.compat_enabled")
        .await
        .map_err(internal)?
        .as_deref()
        == Some("true");
    let expected_hash = settings::get_value(&state.db, "api.key_hash")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    let actual = headers
        .get("X-Api-Key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| query_param(uri, "apikey"))
        .unwrap_or_default();
    if !enabled {
        return Err((
            StatusCode::NOT_FOUND,
            "Overseerr compatibility is disabled".into(),
        ));
    }
    if expected_hash.is_empty()
        || actual.is_empty()
        || settings::hash_api_key(&actual) != expected_hash
    {
        return Err((StatusCode::UNAUTHORIZED, "Invalid API key".into()));
    }
    Ok(())
}
async fn system(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    kind: &str,
) -> Result<Json<Value>, (StatusCode, String)> {
    authorize(state, &headers, &uri).await?;
    Ok(Json(
        json!({"appName":kind,"instanceName":"Oberiz","version":env!("CARGO_PKG_VERSION"),"isDebug":false,"isProduction":true,"startupPath":"/","appData":"/"}),
    ))
}
pub async fn radarr_status(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Value>, (StatusCode, String)> {
    system(&s, h, uri, "Radarr").await
}
pub async fn sonarr_status(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Value>, (StatusCode, String)> {
    system(&s, h, uri, "Sonarr").await
}
async fn quality(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    kind: &str,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    authorize(state, &headers, &uri).await?;
    let rows = profiles::list_profiles_internal(&state.db, kind)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.1))?;
    Ok(Json(rows.into_iter().map(|p|json!({"id":p.id,"name":p.name,"cutoff":p.cutoff_score,"upgradeAllowed":p.upgrade_allowed})).collect()))
}
pub async fn radarr_profiles(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    quality(&s, h, uri, "movie").await
}
pub async fn sonarr_profiles(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    quality(&s, h, uri, "series").await
}
async fn roots(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    key: &str,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    authorize(state, &headers, &uri).await?;
    let path = settings::get_value(&state.db, key)
        .await
        .map_err(internal)?
        .unwrap_or_default();
    Ok(Json(vec![
        json!({"id":1,"path":path,"accessible":!path.trim().is_empty(),"freeSpace":0,"unmappedFolders":[]}),
    ]))
}
pub async fn radarr_roots(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    roots(&s, h, uri, "paths.movies").await
}
pub async fn sonarr_roots(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    roots(&s, h, uri, "paths.series").await
}
pub async fn tags(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    authorize(&s, &h, &uri).await?;
    Ok(Json(vec![]))
}
pub async fn language_profiles(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    authorize(&s, &h, &uri).await?;
    let profiles = profiles::list_language_profiles_internal(&s.db).await?;
    Ok(Json(
        profiles
            .into_iter()
            .map(|profile| {
                json!({
                    "id":profile.id,
                    "name":profile.name,
                    "upgradeAllowed":true,
                    "languages":profile.allowed_languages
                })
            })
            .collect(),
    ))
}
async fn existing(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    kind: &str,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    authorize(state, &headers, &uri).await?;
    let tmdb_id = query_param(&uri, "tmdbId").and_then(|value| value.parse::<i64>().ok());
    let rows: Vec<(i64, i64, String, bool)> = if kind == "movie" {
        if let Some(tmdb_id) = tmdb_id {
            sqlx::query_as("SELECT id,tmdb_id,title,monitored FROM movies WHERE tmdb_id=?")
                .bind(tmdb_id)
                .fetch_all(&state.db)
                .await
                .map_err(internal)?
        } else {
            sqlx::query_as("SELECT id,tmdb_id,title,monitored FROM movies ORDER BY id")
                .fetch_all(&state.db)
                .await
                .map_err(internal)?
        }
    } else {
        if let Some(tmdb_id) = tmdb_id {
            sqlx::query_as("SELECT id,tmdb_id,name,monitored FROM series WHERE tmdb_id=?")
                .bind(tmdb_id)
                .fetch_all(&state.db)
                .await
                .map_err(internal)?
        } else {
            sqlx::query_as("SELECT id,tmdb_id,name,monitored FROM series ORDER BY id")
                .fetch_all(&state.db)
                .await
                .map_err(internal)?
        }
    };
    Ok(Json(rows.into_iter().map(|(id,tmdb,title,monitored)|json!({"id":id,"tmdbId":tmdb,"title":title,"monitored":monitored})).collect()))
}
pub async fn radarr_movies(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    existing(&s, h, uri, "movie").await
}
pub async fn radarr_lookup(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    authorize(&s, &h, &uri).await?;
    // Overseerr asks Radarr to resolve `term=tmdb:<id>` before submitting a movie.
    // It already holds the authoritative title/year and sends them in the following POST.
    let tmdb_id = query_param(&uri, "term")
        .as_deref()
        .and_then(|term| term.strip_prefix("tmdb:")?.parse::<i64>().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "lookup requires term=tmdb:<id>".into(),
            )
        })?;
    Ok(Json(vec![json!({
        "tmdbId":tmdb_id,
        "title":format!("TMDB {tmdb_id}"),
        "year":null,
        "images":[],
        "minimumAvailability":"released",
        "monitored":true
    })]))
}
pub async fn sonarr_lookup(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    authorize(&s, &h, &uri).await?;
    let term = query_param(&uri, "term").unwrap_or_default();
    let tvdb_id = term
        .strip_prefix("tvdb:")
        .and_then(|value| value.parse::<i64>().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "lookup requires term=tvdb:<id>".into(),
            )
        })?;
    Ok(Json(vec![json!({
        "tvdbId":tvdb_id,
        "title":format!("TVDB {tvdb_id}"),
        "year":null,
        "images":[],
        "seasons":[],
        "monitored":true
    })]))
}
pub async fn sonarr_series(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    existing(&s, h, uri, "series").await
}
async fn tmdb_series_id_from_tvdb(
    state: &AppState,
    tvdb_id: i64,
) -> Result<i64, (StatusCode, String)> {
    let credential = settings::get_value(&state.db, "tmdb.api_key")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if credential.trim().is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "TMDB is required to import Sonarr series".into(),
        ));
    }
    let response = crate::tmdb::apply_tmdb_auth(
        state
            .http
            .get(format!("https://api.themoviedb.org/3/find/{tvdb_id}"))
            .query(&[("external_source", "tvdb_id")]),
        &credential,
    )
    .send()
    .await
    .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB returned {} while resolving TVDB", response.status()),
        ));
    }
    let data = response.json::<Value>().await.map_err(internal)?;
    data.pointer("/tv_results/0/id")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("No TMDB series found for TVDB {tvdb_id}"),
            )
        })
}
async fn add(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    body: Value,
    kind: &str,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    authorize(state, &headers, &uri).await?;
    let tmdb = match body
        .get("tmdbId")
        .or_else(|| body.get("tmdb_id"))
        .and_then(Value::as_i64)
    {
        Some(id) => id,
        None if kind == "series" => {
            let tvdb_id = body
                .get("tvdbId")
                .or_else(|| body.get("tvdb_id"))
                .and_then(Value::as_i64)
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        "Sonarr series requires tvdbId".into(),
                    )
                })?;
            tmdb_series_id_from_tvdb(state, tvdb_id).await?
        }
        None => return Err((StatusCode::BAD_REQUEST, "tmdbId is required".into())),
    };
    let profile = body.get("qualityProfileId").and_then(Value::as_i64);
    let monitored = body
        .get("monitored")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let request = CreateRequest {
        media_type: kind.into(),
        tmdb_id: tmdb,
        quality_profile_id: profile,
        client_request_id: None,
        client_name: None,
        monitored: Some(monitored),
        monitor_mode: Some("all".into()),
        requested_seasons: None,
        monitor_future_seasons: None,
        requested_by: Some("Overseerr".into()),
    };
    let view = match public_api::create_request_internal(state, request).await {
        Ok(view) => view,
        // Overseerr already supplies enough movie metadata to create a wanted item.
        // Do not discard the request solely because TMDB's detail endpoint is unavailable.
        Err((StatusCode::BAD_GATEWAY, _)) if kind == "movie" => {
            let title=body.get("title").and_then(Value::as_str).filter(|value|!value.trim().is_empty()).ok_or_else(||(StatusCode::BAD_GATEWAY,"TMDB could not provide movie details and Overseerr did not provide a title".into()))?;
            let year = body
                .get("year")
                .and_then(Value::as_i64)
                .and_then(|value| i32::try_from(value).ok());
            let root = body
                .get("rootFolderPath")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty());
            sqlx::query("INSERT OR IGNORE INTO movies(tmdb_id,title,original_title,year,overview,monitored,quality_profile_id,library_path) VALUES(?,?,?,?,?,?,?,?)")
                .bind(tmdb).bind(title).bind(title).bind(year).bind("").bind(monitored).bind(profile).bind(root)
                .execute(&state.db).await.map_err(internal)?;
            public_api::create_request_internal(
                state,
                CreateRequest {
                    media_type: "movie".into(),
                    tmdb_id: tmdb,
                    quality_profile_id: profile,
                    client_request_id: None,
                    client_name: None,
                    monitored: Some(monitored),
                    monitor_mode: Some("all".into()),
                    requested_seasons: None,
                    monitor_future_seasons: None,
                    requested_by: Some("Overseerr".into()),
                },
            )
            .await?
        }
        Err(error) => return Err(error),
    };
    Ok((
        StatusCode::CREATED,
        Json(
            json!({"id":view.request.media_id,"tmdbId":tmdb,"title":view.title,"monitored":monitored}),
        ),
    ))
}
pub async fn radarr_add_movie(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
    Json(b): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    add(&s, h, uri, b, "movie").await
}
pub async fn sonarr_add_series(
    State(s): State<AppState>,
    h: HeaderMap,
    uri: Uri,
    Json(b): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    add(&s, h, uri, b, "series").await
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[cfg(test)]
mod query_param_tests {
    use super::query_param;
    use axum::http::Uri;

    #[test]
    fn reads_a_plain_value() {
        let uri: Uri = "/api/v3/movie/lookup?term=tmdb:603".parse().unwrap();
        assert_eq!(query_param(&uri, "term").as_deref(), Some("tmdb:603"));
    }

    #[test]
    fn decodes_a_percent_encoded_colon() {
        // A standard URL encoder (Overseerr's own HTTP client included)
        // percent-encodes ':' as the query value is built; this must decode
        // back to the same thing a literal ':' would.
        let uri: Uri = "/api/v3/movie/lookup?term=tmdb%3A603".parse().unwrap();
        assert_eq!(query_param(&uri, "term").as_deref(), Some("tmdb:603"));
    }

    #[test]
    fn returns_none_for_a_missing_key() {
        let uri: Uri = "/api/v3/movie/lookup?other=1".parse().unwrap();
        assert_eq!(query_param(&uri, "term"), None);
    }
}
