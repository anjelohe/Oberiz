use super::*;
use std::sync::Arc;

#[test]
fn normalization_preserves_accents_years_and_episode_markers() {
    assert_eq!(
        without_query_punctuation("El cuervo: Salvación 2000").as_deref(),
        Some("El cuervo Salvación 2000")
    );
    assert_eq!(
        without_query_punctuation("Serie: especial S02E03 2020").as_deref(),
        Some("Serie especial S02E03 2020")
    );
    assert_eq!(
        without_query_punctuation("Salvacio\u{301}n: final").as_deref(),
        Some("Salvacio\u{301}n final")
    );
    assert_eq!(without_query_punctuation("El nuevo Karate Kid 1994"), None);
    assert_eq!(without_query_punctuation("  Plain   title 2020 "), None);
    assert_eq!(without_query_punctuation("... : !!!"), None);
}

type Fixture = (
    Definition,
    Client,
    SearchContext,
    Arc<Mutex<Vec<String>>>,
    tokio::task::JoinHandle<()>,
);

async fn fixture(mode: u8, query: &str) -> Fixture {
    use axum::{Router, extract::Query, response::Html, routing::get};
    let calls = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new().route("/torrents", get({
        let calls = calls.clone();
        move |Query(query): Query<HashMap<String,String>>| {
            let calls = calls.clone();
            async move {
                let name = query.get("name").cloned().unwrap_or_default();
                calls.lock().unwrap().push(name.clone());
                let row = |id: &str, title: &str| format!("<tr class='release'><td><a class='title' href='/details/{id}'>{title}</a><a class='download' href='/download/{id}'>Get</a></td></tr>");
                if mode == 1 && !name.contains(':') {
                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,Html(String::new()));
                }
                let body = if mode == 2 {
                    if name == "El cuervo Salvación" { row("found","El cuervo Salvacion 2000 WEB-DL 1080p") } else { String::new() }
                } else {
                    format!("{}{}",row("shared","El cuervo Salvacion 2000 1080p"),if name.contains(':') { row("original","Original copy 2000 REMUX 1080p") } else { row("better","Better copy 2000 WEB-DL 1080p") })
                };
                (axum::http::StatusCode::OK,Html(format!("<html><table>{body}</table></html>")))
            }
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let definition = Definition {
        id: "fixture".into(),
        name: "Fixture".into(),
        base_url: format!("http://{address}/"),
        config: Default::default(),
        yaml: serde_yaml::from_str(
            r#"
search:
  path: torrents
  inputs:
    name: "{{ .Keywords }}"
  rows:
    selector: tr.release
  fields:
    title:
      selector: a.title
    details:
      selector: a.title
      attribute: href
    download:
      selector: a.download
      attribute: href
    size:
      text: 5 GB
    seeders:
      text: 7
"#,
        )
        .unwrap(),
    };
    let context = SearchContext {
        keywords: query.into(),
        alternate_keywords: None,
        tmdb_id: None,
        imdb_id: None,
        tvdb_id: None,
        media_type: "movie".into(),
        year: Some(2000),
        season: None,
        episode: None,
    };
    (
        definition,
        Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        context,
        calls,
        server,
    )
}

#[tokio::test]
async fn normalized_search_runs_even_when_original_has_results_and_merges_without_duplicates() {
    let (definition, client, context, calls, server) =
        fixture(0, "El cuervo: Salvación 2000").await;
    let rows = search_query_variants(&definition, &client, &context, &context.keywords)
        .await
        .unwrap();
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["El cuervo: Salvación 2000", "El cuervo Salvación 2000"]
    );
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().any(|r| r.title.starts_with("Better copy")));
    assert!(rows.iter().any(|r| r.title.starts_with("Original copy")));
    assert_eq!(
        rows.iter()
            .filter(|r| r.title.starts_with("El cuervo"))
            .count(),
        1
    );
    server.abort();
}

#[tokio::test]
async fn unchanged_query_is_requested_once() {
    let (definition, client, context, calls, server) = fixture(0, "El nuevo Karate Kid 1994").await;
    let rows = search_query_variants(&definition, &client, &context, &context.keywords)
        .await
        .unwrap();
    assert_eq!(calls.lock().unwrap().len(), 1);
    assert_eq!(rows.len(), 2);
    server.abort();
}

#[tokio::test]
async fn failure_of_extra_variant_preserves_original_hits() {
    let (definition, client, context, calls, server) =
        fixture(1, "El cuervo: Salvación 2000").await;
    let rows = search_query_variants(&definition, &client, &context, &context.keywords)
        .await
        .unwrap();
    assert_eq!(calls.lock().unwrap().len(), 2);
    assert_eq!(rows.len(), 2);
    server.abort();
}

#[tokio::test]
async fn year_fallback_is_preserved_for_both_variants_and_bounded() {
    let (definition, client, context, calls, server) =
        fixture(2, "El cuervo: Salvación 2000").await;
    let rows = search_query_variants(&definition, &client, &context, &context.keywords)
        .await
        .unwrap();
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            "El cuervo: Salvación 2000",
            "El cuervo: Salvación",
            "El cuervo Salvación 2000",
            "El cuervo Salvación"
        ]
    );
    assert_eq!(rows.len(), 1);
    server.abort();
}
