//! Turning a release title into a filesystem-safe name, and pulling season/
//! episode numbers back out of a file name for linking against the library.
use crate::releases;

pub(super) fn parse_episode_numbers(name: &str) -> Option<(i32, i32, Option<i32>)> {
    // `\b` after the optional second episode number matters: without it, a
    // release like "S01E02.1080p.mkv" let the digit run in "1080p" get
    // captured as a bogus second episode ("2 to 108"), which would then mark
    // over a hundred episodes as having a file.
    let patterns = [
        r"(?i)S(\d{1,2})E(\d{1,3})(?:[-_. ]?E?(\d{1,3})\b)?",
        r"(?i)(\d{1,2})x(\d{1,3})(?:[-_. ]?(\d{1,3})\b)?",
    ];
    for pattern in patterns {
        let re = regex::Regex::new(pattern).ok()?;
        if let Some(caps) = re.captures(name) {
            let season = caps.get(1)?.as_str().parse().ok()?;
            let first = caps.get(2)?.as_str().parse().ok()?;
            let last = caps.get(3).and_then(|m| m.as_str().parse().ok());
            return Some((season, first, last));
        }
    }
    None
}

pub(super) fn parse_absolute_episode(name: &str) -> Option<i32> {
    let re = regex::Regex::new(r"(?i)(?:^|[ ._\-])(?:EP?|EPISODE)[ ._\-]?(\d{1,3})(?:[ ._\-]|$)")
        .ok()?;
    re.captures(name)?.get(1)?.as_str().parse().ok()
}

pub(super) fn render_series_template(
    template: &str,
    title: &str,
    year: Option<i32>,
    parsed: &releases::ParsedRelease,
    season: i32,
    episode: i32,
    episode_title: &str,
) -> String {
    let mut value = render_template(template, title, year, parsed)
        .replace("{Season}", &season.to_string())
        .replace("{Episode}", &episode.to_string())
        .replace("{Season:00}", &format!("{season:02}"))
        .replace("{Episode:00}", &format!("{episode:02}"))
        .replace("{EpisodeTitle}", episode_title);
    while value.contains("  ") {
        value = value.replace("  ", " ");
    }
    value.trim().trim_matches('-').trim().to_string()
}

pub(super) fn render_template(
    template: &str,
    title: &str,
    year: Option<i32>,
    parsed: &releases::ParsedRelease,
) -> String {
    let mut value = template
        .replace("{Title}", title)
        .replace("{Year}", &year.map(|v| v.to_string()).unwrap_or_default())
        .replace("{Resolution}", parsed.resolution.as_deref().unwrap_or(""))
        .replace("{Source}", parsed.source.as_deref().unwrap_or(""))
        .replace("{Codec}", parsed.codec.as_deref().unwrap_or(""))
        .replace("{HDR}", parsed.hdr.as_deref().unwrap_or(""))
        .replace("{Audio}", parsed.audio.as_deref().unwrap_or(""))
        .replace("{Language}", parsed.language.as_deref().unwrap_or(""));
    while value.contains("  ") {
        value = value.replace("  ", " ");
    }
    value.trim().trim_matches('-').trim().to_string()
}

pub(super) fn sanitize_name(value: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    value
        .chars()
        .map(|c| if invalid.contains(&c) { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_name_strips_path_separators_and_reserved_characters() {
        assert_eq!(
            sanitize_name("../../etc/passwd:  weird | name?"),
            ".. .. etc passwd weird name"
        );
        assert_eq!(sanitize_name("Normal Title (2024)"), "Normal Title (2024)");
        assert_eq!(sanitize_name("  extra   spaces  "), "extra spaces");
    }

    #[test]
    fn parse_episode_numbers_recognizes_common_formats() {
        assert_eq!(
            parse_episode_numbers("Show.Name.S01E02.1080p.mkv"),
            Some((1, 2, None))
        );
        assert_eq!(
            parse_episode_numbers("Show Name 1x02 HDTV"),
            Some((1, 2, None))
        );
        assert_eq!(
            parse_episode_numbers("Show.Name.S02E05-E06.WEB.mkv"),
            Some((2, 5, Some(6)))
        );
        assert_eq!(parse_episode_numbers("Show.Name.Complete.Season.mkv"), None);
    }

    #[test]
    fn render_template_fills_known_tokens_and_collapses_whitespace() {
        let parsed = releases::parse("Movie.Title.2024.2160p.WEB-DL.DDP5.1.HDR.x265-GROUP");
        let name = render_template(
            "{Title} ({Year}) - {Resolution} {Source} {Codec}",
            "Movie Title",
            Some(2024),
            &parsed,
        );
        assert_eq!(name, "Movie Title (2024) - 2160P WEB-DL x265");
    }

    #[test]
    fn render_series_template_fills_season_and_episode_tokens() {
        let parsed = releases::parse("Show.Name.S01E02.1080p.WEB.x264-GROUP");
        let name = render_series_template(
            "{Title} - S{Season:00}E{Episode:00} - {EpisodeTitle}",
            "Show Name",
            None,
            &parsed,
            1,
            2,
            "Pilot",
        );
        assert_eq!(name, "Show Name - S01E02 - Pilot");
    }
}
