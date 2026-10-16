# Oberiz security audit — October 14, 2026

## Status after corrections

v1.0.3 update: initial password setup no longer requires loopback. Configure initially on a trusted network. Manual qBittorrent file deletion is allowed with confirmation and an unchecked checkbox; automatic policies and rejection preserve data. The sections below document the original audit and initial measures, not those later revisions.

The findings below describe the original audit. Subsequently added:

- Protection of Windows private directories and their files; Unix configuration/data permissions 0700. Owner/service identity preserved. No following links/reparse points or changing NAS permissions.
- Initial password restricted to direct loopback without proxies, and serialized password changes to prevent initialization races (loopback restriction subsequently removed in v1.0.3).
- Native/compatibility API authentication before request bodies and at most four concurrent integration requests.
- rustls 0.23.46, corrected source-map-js, updated scraper without fxhash, and a maintained YAML parser replacing serde_yaml/unsafe-libyaml. Catalog verified.
- Active-dependency OSV and npm audits in CI/releases.
- CSP and configurable listening address through OBERIZ_BIND_ADDRESS.
- LocalService for new Windows services. Upgrades preserve existing accounts; migration requires reviewing installation-specific permissions.

Installation/operational verification remains necessary: code changes alone do not replace the installed executable. ACLs were completed after user-approved elevation: the helper reported `success`; recursive inspection found no Users, Everyone, or Authenticated Users rules in data/configuration, including DB, WAL, SHM, and backups. Previous ACLs are retained.

No proxy, certificate, firewall, existing service account, credential, or media-disk permission changes. Connection credentials remain reversible and permission-protected: SQLite is not fully encrypted, nor can backups exported elsewhere be protected retroactively.

See the [security guide](Oberiz_Seguridad.md) for installation and limitations.

## Scope and result

Review of local code, including unpublished changes relative to commit 315711c: authentication, public API, external library, credentials, filesystem access, qBittorrent operations, backups, packaging, and dependencies. Windows-installation ACL inspection without reading or printing credentials. During the original review there were no code/configuration/permission changes or torrent/NAS operations.

This report is not a certification or a complete penetration test. Risks required correction before exposing Oberiz directly to the Internet.

## Priority findings

### SEC-01 — High: credentials accessible to other local users

Confirmed: `%ProgramData%\Oberiz\data\oberiz.db` and the backup directory inherited read permissions for `BUILTIN\Users` (localized as Usuarios). Data/config/backups directories also inherited write permissions whose effective scope required review during ACL remediation.

`settings.rs:441-450` stored TMDB/TVDB keys and the qBittorrent password unencrypted; `indexers.rs:762` stored indexer credentials in JSON. The administrator password and public API key were hashed. SQLite backups contain the same sensitive data.

The SMB password used DPAPI with `CRYPTPROTECT_LOCAL_MACHINE` (`smb.rs:251`). This is not per-user protection: another user on the same computer who obtains the blob can decrypt it. No real data was decrypted.

Recommendation: restrict data/configuration/backup and existing-file ACLs to the service, administrators, and appropriate owner. Protect WAL/SHM and check read/write access from an ordinary account. Consider encrypting credentials/backups without colocated keys.

Reference: [Microsoft, CryptProtectData](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata).

### SEC-02 — Conditional high: initial-setup takeover

`auth.rs:347` allowed `/api/auth/password` while no password existed. `auth.rs:431` required neither an installation secret nor a local client. `main.rs:278` listened on `0.0.0.0:2032`.

On a new network-exposed installation, the first visitor can set a password and take administrator access. An initialization race also existed because checking/saving was not atomic. This does not bypass an already-configured password.

Recommendation at the time: local initialization or a one-use token, an atomic operation, and restricted listening/firewall during first startup. The installed service's setup state was not checked because the local query did not respond.

### SEC-03 — Medium: memory/CPU consumption before API authentication

`routes/public.rs:31` accepted library bodies up to 128 MiB. `external_library.rs:120` used the JSON extractor before `authorize`. The route bypassed the administrator session and relied on handler validation. A client without a key could trigger receipt/deserialization of a large body before rejection, even with the API disabled.

Recommendation: authenticate in middleware/a pre-body extractor, apply endpoint limits and concurrency/rate limiting. Code confirmed ordering; no destructive load test or failure-threshold measurement was performed.

### SEC-04 — Dependencies: one npm advisory and seven OSV advisories in Cargo.lock

`npm audit` detected `source-map-js 1.2.1`, transitive through Vite/PostCSS, with a high-severity denial-of-service advisory when processing source maps. Fixed in 1.2.2. Primary exposure here is the build chain: the published frontend is static, not a service accepting visitor source maps. Remote exploitation of installed Oberiz was not demonstrated.

Source: [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q).

330 Cargo.lock package/version pairs were checked with OSV:

- `rustls 0.23.45`: RUSTSEC-2026-0346/0347/0348/0349; fixed in 0.23.46. Exposure depends on configuration: Brotli was not enabled in the reviewed tree; no explicit ECH or custom TLS cache with secret logging was found. The observed provider was ring. Exploitability of all four issues under Oberiz configuration was not demonstrated.
- `rsa 0.9.10`: RUSTSEC-2023-0071. Present in the lockfile but not the queried active Windows-target tree. Not classified as a demonstrated active vulnerability in this build.
- `unsafe-libyaml 0.2.11`: RUSTSEC-2024-0449, unmaintained; introduced through serde_yaml. Informational advisory, not confirmed exploitation.
- `fxhash 0.2.1`: RUSTSEC-2025-0057, unmaintained; informational advisory.

Sources: [OSV 0346](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0346), [OSV 0347](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0347), [OSV 0348](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0348), [OSV 0349](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0349), [RSA](https://rustsec.org/advisories/RUSTSEC-2023-0071.html), [unsafe-libyaml](https://rustsec.org/advisories/RUSTSEC-2024-0449.html), [fxhash](https://rustsec.org/advisories/RUSTSEC-2025-0057.html).

Recommendation: upgrade to fixed versions while retaining tests/lockfiles; plan replacements for abandoned dependencies. Add dependency audits to CI: Clippy and ordinary tests do not query advisories. cargo-audit was not installed; OSV and cargo tree were used.

## Additional hardening

- Windows service installed as LocalSystem (`service.rs:147`), increasing intrusion impact. Consider a dedicated least-privilege identity without breaking SMB or existing installations.
- HTTP listened on all interfaces. Terminate TLS at a proxy, with `OBERIZ_COOKIE_SECURE=true`, firewall, and explicit trusted proxies. Current firewall/proxy/public exposure was not verified.
- Integration key is shared and unscoped: holders can send authorized reports/requests, not merely read. Separate library/request permissions and anomalous-change logging are recommended.
- External paths are information, not Cinetta access. `superseded` does not delete anything and must not become automatic Cinetta deletion without independent validation, especially for multi-episode files.

## Verified protections

- Argon2 passwords, random 256-bit sessions stored as hashes, revocation, and HttpOnly/SameSite=Strict cookies.
- Login-attempt and concurrent Argon2-work limiting.
- Public API with a hashed key, disable switch, and separation from web sessions.
- No SMB passwords or keys returned by reviewed normal responses.
- qBittorrent file deletion blocked in external mode, both manually and in the internal rejection/cleanup implementation (later changed for explicitly confirmed manual deletion).
- Cinetta library stored as virtual paths; reports neither open nor modify files. Protection for offline disks and double episodes.
- Parameterized queries and backup-name/path validation; indexer-download defense against private destinations and redirects.
- Restoring a backup does not restore revoked old sessions.
- Pinned Actions and Docker bases; pinning does not replace updates.

## Verification and limitations

100 unit tests and one migration test passed; the optional real-disk test was not run. No secret reading, key transmission to third parties, password changes, exploitation, load tests, torrent deletion, or NAS access. Real ACLs and current advisories were inspected, not complete NAS/Cinetta security, firewall, or every TLS platform/feature.

Suggested order: ACLs/credentials → secure first startup → authentication before JSON → dependency updates → least privilege and CI audits.
