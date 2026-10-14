use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

// Real release names separate tags with dots or hyphens, not spaces
// ("The.Movie.2024.DV.HDR10.x265"), so this needs word boundaries rather
// than the literal " DV " the naive check used to require.
static DOLBY_VISION: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\bDOLBY[.\s_-]?VISION\b|\bDOVI\b|\bDV\b").unwrap());

#[derive(Debug, Deserialize)]
pub struct ParseQuery {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ParsedRelease {
    pub name: String,
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub codec: Option<String>,
    pub hdr: Option<String>,
    pub audio: Option<String>,
    pub language: Option<String>,
    pub remux: bool,
    pub proper: bool,
    pub repack: bool,
    pub score: i32,
}

pub async fn parse_release(Query(q): Query<ParseQuery>) -> Json<ParsedRelease> {
    Json(parse(&q.name))
}

pub fn parse(name: &str) -> ParsedRelease {
    let upper = name.to_uppercase();

    let resolution = ["2160P", "1080P", "720P", "480P"]
        .iter()
        .find(|x| upper.contains(**x))
        .map(|x| x.to_string());

    let source = [
        ("REMUX", "REMUX"),
        ("BLURAY", "BluRay"),
        ("BLU-RAY", "BluRay"),
        ("WEB-DL", "WEB-DL"),
        ("WEBDL", "WEB-DL"),
        ("WEBRIP", "WEBRip"),
        ("HDTV", "HDTV"),
        ("CAM", "CAM"),
        ("TELESYNC", "TS"),
    ]
    .iter()
    .find(|(needle, _)| upper.contains(needle))
    .map(|(_, value)| value.to_string());

    let codec = [
        ("X265", "x265"),
        ("H265", "HEVC"),
        ("HEVC", "HEVC"),
        ("X264", "x264"),
        ("H264", "H.264"),
        ("AV1", "AV1"),
    ]
    .iter()
    .find(|(needle, _)| upper.contains(needle))
    .map(|(_, value)| value.to_string());

    let hdr = if DOLBY_VISION.is_match(&upper) {
        Some("Dolby Vision".to_string())
    } else if upper.contains("HDR10+") {
        Some("HDR10+".to_string())
    } else if upper.contains("HDR") {
        Some("HDR".to_string())
    } else {
        None
    };

    let audio = [
        ("TRUEHD", "TrueHD"),
        ("ATMOS", "Atmos"),
        ("DTS-HD", "DTS-HD"),
        ("DTS", "DTS"),
        ("DDP", "DD+"),
        ("EAC3", "E-AC3"),
        ("FLAC", "FLAC"),
        ("AAC", "AAC"),
    ]
    .iter()
    .find(|(needle, _)| upper.contains(needle))
    .map(|(_, value)| value.to_string());

    let language =
        if upper.contains("CASTELLANO") || upper.contains("SPANISH") || upper.contains("ES-ES") {
            Some("Spanish".to_string())
        } else if upper.contains("LATINO") || upper.contains("LATIN SPANISH") {
            Some("Latino".to_string())
        } else if upper.contains("DUAL") {
            Some("Dual".to_string())
        } else if upper.contains("MULTI") || upper.contains("MULTILANG") {
            Some("Multi".to_string())
        } else if upper.contains("ENGLISH") || upper.contains(" ENG ") {
            Some("English".to_string())
        } else {
            None
        };

    let remux = upper.contains("REMUX");
    let proper = upper.contains("PROPER");
    let repack = upper.contains("REPACK");

    let mut score = 0;
    score += match resolution.as_deref() {
        Some("2160P") => 400,
        Some("1080P") => 250,
        Some("720P") => 100,
        _ => 0,
    };
    score += match source.as_deref() {
        Some("REMUX") => 180,
        Some("BluRay") => 130,
        Some("WEB-DL") => 100,
        Some("WEBRip") => 60,
        Some("CAM") | Some("TS") => -1000,
        _ => 0,
    };
    if hdr.as_deref() == Some("Dolby Vision") {
        score += 80;
    } else if hdr.is_some() {
        score += 40;
    }
    if matches!(codec.as_deref(), Some("HEVC") | Some("x265") | Some("AV1")) {
        score += 30;
    }
    if matches!(audio.as_deref(), Some("TrueHD") | Some("Atmos")) {
        score += 35;
    }
    if proper || repack {
        score += 10;
    }

    ParsedRelease {
        name: name.to_string(),
        resolution,
        source,
        codec,
        hdr,
        audio,
        language,
        remux,
        proper,
        repack,
        score,
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn scores_a_high_quality_release_above_a_cam() {
        let premium = parse("Film.2026.2160p.BluRay.REMUX.DV.TrueHD.Atmos.SPANISH");
        let cam = parse("Film.2026.1080p.CAM.SPANISH");
        assert_eq!(premium.resolution.as_deref(), Some("2160P"));
        assert_eq!(premium.source.as_deref(), Some("REMUX"));
        assert!(premium.score > cam.score);
        assert!(cam.score < 0);
    }

    #[test]
    fn parses_language_and_release_flags() {
        let parsed = parse("Serie.S01E02.1080p.WEB-DL.H264.DUAL.PROPER.REPACK");
        assert_eq!(parsed.language.as_deref(), Some("Dual"));
        assert!(parsed.proper);
        assert!(parsed.repack);
        assert_eq!(parsed.source.as_deref(), Some("WEB-DL"));
    }

    /// A corpus of real-world-shaped release names with the exact fields
    /// `parse` is expected to extract from each. Every parsing bug found from
    /// here on should get a new row instead of a one-off fix, so the corpus
    /// only grows and a fix can never silently regress later.
    struct Expected {
        resolution: Option<&'static str>,
        source: Option<&'static str>,
        codec: Option<&'static str>,
        hdr: Option<&'static str>,
        audio: Option<&'static str>,
        language: Option<&'static str>,
        remux: bool,
        proper: bool,
        repack: bool,
    }

    #[test]
    fn parses_a_corpus_of_real_release_names() {
        let cases: &[(&str, Expected)] = &[
            (
                "The.Matrix.1999.2160p.UHD.BluRay.REMUX.HDR10.DV.TrueHD.Atmos.7.1-GROUP",
                Expected {
                    resolution: Some("2160P"),
                    source: Some("REMUX"),
                    codec: None,
                    hdr: Some("Dolby Vision"),
                    audio: Some("TrueHD"),
                    language: None,
                    remux: true,
                    proper: false,
                    repack: false,
                },
            ),
            (
                "Interstellar.2014.1080p.BluRay.x264.DTS-HD.MA.5.1-GROUP",
                Expected {
                    resolution: Some("1080P"),
                    source: Some("BluRay"),
                    codec: Some("x264"),
                    hdr: None,
                    audio: Some("DTS-HD"),
                    language: None,
                    remux: false,
                    proper: false,
                    repack: false,
                },
            ),
            (
                "Dune.Part.Two.2024.2160p.WEB-DL.DDP5.1.Atmos.HDR10+.HEVC-GROUP",
                Expected {
                    resolution: Some("2160P"),
                    source: Some("WEB-DL"),
                    codec: Some("HEVC"),
                    hdr: Some("HDR10+"),
                    audio: Some("Atmos"),
                    language: None,
                    remux: false,
                    proper: false,
                    repack: false,
                },
            ),
            (
                "Show.Name.S03E07.720p.HDTV.x264-GROUP",
                Expected {
                    resolution: Some("720P"),
                    source: Some("HDTV"),
                    codec: Some("x264"),
                    hdr: None,
                    audio: None,
                    language: None,
                    remux: false,
                    proper: false,
                    repack: false,
                },
            ),
            (
                "Pelicula.Castellano.2023.1080p.WEBRip.AAC-GROUP",
                Expected {
                    resolution: Some("1080P"),
                    source: Some("WEBRip"),
                    codec: None,
                    hdr: None,
                    audio: Some("AAC"),
                    language: Some("Spanish"),
                    remux: false,
                    proper: false,
                    repack: false,
                },
            ),
            (
                "Show.Name.S02E10.PROPER.1080p.WEB.h264-GROUP",
                Expected {
                    resolution: Some("1080P"),
                    source: None,
                    codec: Some("H.264"),
                    hdr: None,
                    audio: None,
                    language: None,
                    remux: false,
                    proper: true,
                    repack: false,
                },
            ),
            (
                "Movie.Title.2022.CAM.LATINO",
                Expected {
                    resolution: None,
                    source: Some("CAM"),
                    codec: None,
                    hdr: None,
                    audio: None,
                    language: Some("Latino"),
                    remux: false,
                    proper: false,
                    repack: false,
                },
            ),
            (
                "Show.Name.S01.COMPLETE.MULTI.720p.WEB-DL.AV1-GROUP",
                Expected {
                    resolution: Some("720P"),
                    source: Some("WEB-DL"),
                    codec: Some("AV1"),
                    hdr: None,
                    audio: None,
                    language: Some("Multi"),
                    remux: false,
                    proper: false,
                    repack: false,
                },
            ),
        ];

        for (name, expected) in cases {
            let parsed = parse(name);
            assert_eq!(
                parsed.resolution.as_deref(),
                expected.resolution,
                "resolution for {name}"
            );
            assert_eq!(
                parsed.source.as_deref(),
                expected.source,
                "source for {name}"
            );
            assert_eq!(parsed.codec.as_deref(), expected.codec, "codec for {name}");
            assert_eq!(parsed.hdr.as_deref(), expected.hdr, "hdr for {name}");
            assert_eq!(parsed.audio.as_deref(), expected.audio, "audio for {name}");
            assert_eq!(
                parsed.language.as_deref(),
                expected.language,
                "language for {name}"
            );
            assert_eq!(parsed.remux, expected.remux, "remux for {name}");
            assert_eq!(parsed.proper, expected.proper, "proper for {name}");
            assert_eq!(parsed.repack, expected.repack, "repack for {name}");
        }
    }
}
