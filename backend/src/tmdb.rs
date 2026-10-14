use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, settings};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Debug, Deserialize)]
struct MovieSearchResponse {
    results: Vec<TmdbMovie>,
}

#[derive(Debug, Deserialize)]
struct TmdbMovie {
    id: i64,
    title: String,
    #[serde(default)]
    original_title: String,
    #[serde(default)]
    overview: String,
    release_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    #[serde(default)]
    vote_average: f64,
}

#[derive(Debug, Deserialize)]
struct SeriesSearchResponse {
    results: Vec<TmdbSeries>,
}

#[derive(Debug, Deserialize)]
struct TmdbSeries {
    id: i64,
    name: String,
    #[serde(default)]
    original_name: String,
    #[serde(default)]
    overview: String,
    first_air_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    #[serde(default)]
    vote_average: f64,
}

#[derive(Debug, Serialize)]
pub struct MediaSearchResult {
    pub tmdb_id: i64,
    pub title: String,
    pub original_title: String,
    pub year: Option<i32>,
    pub overview: String,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub vote_average: f64,
}

fn normalize_tmdb_credential(raw: &str) -> String {
    let mut value = raw.trim();

    if let Some(stripped) = value.strip_prefix("Bearer ") {
        value = stripped.trim();
    } else if let Some(stripped) = value.strip_prefix("bearer ") {
        value = stripped.trim();
    }

    // Be forgiving if the credential was pasted with surrounding quotes.
    value
        .trim_matches(|c| c == '"' || c == '\'')
        .trim()
        .to_string()
}

pub fn apply_tmdb_auth(
    request: reqwest::RequestBuilder,
    credential: &str,
) -> reqwest::RequestBuilder {
    let credential = normalize_tmdb_credential(credential);

    if credential.starts_with("eyJ") {
        request.bearer_auth(credential)
    } else {
        request.query(&[("api_key", credential)])
    }
}

async fn credential(state: &AppState) -> Result<String, (StatusCode, String)> {
    let raw = settings::get_value(&state.db, "tmdb.api_key")
        .await
        .map_err(internal_error)?
        .unwrap_or_default();

    let value = normalize_tmdb_credential(&raw);

    if value.is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "TMDB no está configurado".into(),
        ));
    }

    Ok(value)
}

pub async fn search_movies(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<MediaSearchResult>>, (StatusCode, String)> {
    let query = params.query.trim();
    if query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "La búsqueda no puede estar vacía".into(),
        ));
    }

    let credential = credential(&state).await?;
    let response = apply_tmdb_auth(
        state
            .http
            .get("https://api.themoviedb.org/3/search/movie")
            .query(&[
                ("query", query),
                ("language", "es-ES"),
                ("include_adult", "false"),
            ]),
        &credential,
    )
    .send()
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Error conectando con TMDB: {e}"),
        )
    })?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        let mode = if credential.starts_with("eyJ") {
            "Bearer"
        } else {
            "API key v3"
        };
        return Err((
            StatusCode::UNAUTHORIZED,
            format!(
                "TMDB devolvió 401. Oberiz está usando {} (longitud {}).",
                mode,
                credential.len()
            ),
        ));
    }
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB devolvió {}", response.status()),
        ));
    }

    let data = response.json::<MovieSearchResponse>().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Respuesta de TMDB no válida: {e}"),
        )
    })?;

    Ok(Json(
        data.results
            .into_iter()
            .map(|m| MediaSearchResult {
                tmdb_id: m.id,
                title: m.title,
                original_title: m.original_title,
                year: m
                    .release_date
                    .as_deref()
                    .and_then(|d| d.get(0..4))
                    .and_then(|y| y.parse().ok()),
                overview: m.overview,
                poster_url: m
                    .poster_path
                    .map(|p| format!("https://image.tmdb.org/t/p/w500{p}")),
                backdrop_url: m
                    .backdrop_path
                    .map(|p| format!("https://image.tmdb.org/t/p/w1280{p}")),
                vote_average: m.vote_average,
            })
            .collect(),
    ))
}

pub async fn search_series(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<MediaSearchResult>>, (StatusCode, String)> {
    let query = params.query.trim();
    if query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "La búsqueda no puede estar vacía".into(),
        ));
    }

    let credential = credential(&state).await?;
    let response = apply_tmdb_auth(
        state
            .http
            .get("https://api.themoviedb.org/3/search/tv")
            .query(&[
                ("query", query),
                ("language", "es-ES"),
                ("include_adult", "false"),
            ]),
        &credential,
    )
    .send()
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Error conectando con TMDB: {e}"),
        )
    })?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        let mode = if credential.starts_with("eyJ") {
            "Bearer"
        } else {
            "API key v3"
        };
        return Err((
            StatusCode::UNAUTHORIZED,
            format!(
                "TMDB devolvió 401. Oberiz está usando {} (longitud {}).",
                mode,
                credential.len()
            ),
        ));
    }
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB devolvió {}", response.status()),
        ));
    }

    let data = response.json::<SeriesSearchResponse>().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Respuesta de TMDB no válida: {e}"),
        )
    })?;

    Ok(Json(
        data.results
            .into_iter()
            .map(|m| MediaSearchResult {
                tmdb_id: m.id,
                title: m.name,
                original_title: m.original_name,
                year: m
                    .first_air_date
                    .as_deref()
                    .and_then(|d| d.get(0..4))
                    .and_then(|y| y.parse().ok()),
                overview: m.overview,
                poster_url: m
                    .poster_path
                    .map(|p| format!("https://image.tmdb.org/t/p/w500{p}")),
                backdrop_url: m
                    .backdrop_path
                    .map(|p| format!("https://image.tmdb.org/t/p/w1280{p}")),
                vote_average: m.vote_average,
            })
            .collect(),
    ))
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Error de base de datos: {error}"),
    )
}
