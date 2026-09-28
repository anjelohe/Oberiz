use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};

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

    let hdr = if upper.contains("DOLBY.VISION") || upper.contains("DOVI") || upper.contains(" DV ")
    {
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
}
