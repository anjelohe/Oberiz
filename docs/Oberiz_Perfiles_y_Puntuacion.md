# Quality profiles, scoring, and indexer priority

This guide explains how Oberiz chooses which release to show first. Profiles do not automatically download a release merely because it ranks first: they accept, reject, and sort available results.

## 1. Acceptance: the mandatory filter

Before comparing scores, Oberiz evaluates each release against the selected profile. A rejected release always ranks below an accepted release, even when indexer priority is enabled.

Common rejection reasons:

- insufficient title match;
- an incompatible year, when the declared release year differs by more than one year;
- the wrong media type, such as a season when searching for a movie;
- an explicitly different season from the requested one;
- seeders below the minimum;
- size outside the profile limits;
- a blocked term;
- a disallowed resolution, source, or language.

Blocked terms are compared as release tokens. For example, `CAM` blocks `CAM.1080p`, but does not block a title containing those letters inside another word.

## 2. Score calculation

Every accepted release receives a total score:

```text
title match
+ profile score
+ seeder bonus (maximum 60)
```

The profile score adds the configured values for:

- resolution;
- source;
- codec;
- HDR;
- audio;
- preferred terms such as `PROPER` or `REPACK`;
- language, according to the associated language profile;
- a `+35` series-pack bonus when **Prefer season/complete packs** is enabled.

Values are numerical preferences, not mandatory tiers. A configured dimension may also reject disallowed items, such as an unchecked resolution or source. Optional dimensions—codec, HDR, and audio—add points only when recognized and present in the profile.

## 3. Series size limits

Series profiles provide two maximums:

- **Max episode size MB** applies to explicit episodes such as `S03E07`, `3x07`, `Episode`, or the recognized Spanish marker `Episodio`.
- **Max season/pack size MB** applies to `S03`, `Temporada 3`, `PACK`, or `Complete` without an episode marker. The Spanish example is intentional release-parser input.

When the second field is empty, Oberiz also uses the general episode maximum for packs. This preserves previous behavior until a separate maximum is configured.

## 4. Normal ordering: quality first

With **Prioritize indexer** disabled, results are sorted by:

1. Accepted releases before rejected releases.
2. Higher total score.
3. Lower numerical indexer priority.
4. More seeders.

Indexer priority is a tiebreaker in this mode. Use it when the quality profile fully represents your preferences.

## 5. Ordering with Prioritize indexer

Each quality profile can enable **Prioritize indexer**, changing the order to:

1. Accepted releases before rejected releases.
2. Lower numerical indexer priority.
3. Higher total score within that indexer.
4. More seeders.

This favors an indexer that usually publishes better copies while retaining profile acceptance rules. A release violating resolution, language, size, seeders, blocked terms, or another rule is still rejected and cannot win through priority.

## 6. Configuring priority

Set **Search priority** when editing each indexer. It is displayed in the Indexers table.

- Default: `100`.
- Lower numbers are preferred: `10` is queried and sorted before `100`.
- Priority does not add score points; it is an ordering rule.

For example, with indexers at `10` and `100` and **Prioritize indexer** enabled, accepted releases from indexer `10` appear first. Within that group, Oberiz sorts by quality score and then seeders.

## 7. Practical guidance

- Configure acceptance rules and quality points first to exclude unwanted formats or languages.
- Use normal priority for neutral tiebreaking.
- Enable **Prioritize indexer** only for profiles where you particularly trust one or more indexers' editorial choices.
- Leave it disabled when comparing the best available score strictly across all sources.

Private tracker definitions, addresses, credentials, and notes are not part of this guide or repository.
