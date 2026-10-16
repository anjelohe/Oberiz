# Security audit before 1.0.4 — October 16, 2026

## Delivery traceability

Fixes delivered to GitHub on `main`, commit `ca0d1d3`, tag `v1.0.4`, with an atomic push. CI `38929148251` and installers `38929148331` were recorded as running during the initial delivery check. The release was later confirmed available with five assets when its notes were translated. No service deployment or qBittorrent restart. Real memory consumption remains to be checked. Post-delivery documentation does not modify the release tag.

## Follow-up review closure — authorized corrections

The following four findings were fixed for v1.0.4. Allowed aliases share credentials through a client without proxies or automatic redirects, pinned to the validated public address. Login validates each hop, at most five, without arbitrary ports. The explicitly configured local origin remains allowed. Metadata is consumed sequentially without preloading all seasons or creating a task per resource. The global quota protects the refreshed scope's latest valid report and applies only after validating configuration. Episode ranges use checked addition; inventory expansion is limited to 500,000 unique episodes.

Added regressions cover pinned connections with cookies/headers, private login aliases, ports, overflow, and report preservation with a full cache. Initial password setup over the network is unchanged. This audit does not establish real qBittorrent memory consumption; observation in real use remains pending. Historical review sections are retained below.

Final verification: 124 passing unit tests, one ignored; two migration tests; five frontend suites and build; formatting and Clippy passed. Dependencies: 252 active Rust versions without OSV advisories and zero npm vulnerabilities.

## Review of subsequent changes — historical findings, now corrected

The user supplied new unstaged changes on top of 57 staged files and requested review before publication. At that review stage code was not modified and no commit, tag, or release had been created. These findings superseded the earlier closure for subsequently changed files.

1. **P1 — Credential/DNS policy:** in `cardigann.rs`, the new `same_site` branch of `authorize_fetch` checked a public DNS resolution but returned the original authenticated client, which resolved again on connection. This reintroduced DNS rebinding for www variants/subdomains. Login redirects also admitted the www variant without IP validation. Preserve legitimate-host compatibility but pin the validated IP and authorized origin/port before sending cookies or headers.
2. **P1 — Report memory:** `metadata_many` created a task per path and retained all results. `calculate` preloaded all seasons into a BTreeMap before enforcing the 20,000-missing-item limit. Input limits allowed 1,000 series × 200 seasons. Four concurrent requests did not bound retained response memory. Process small batches/release results with total quotas and early stopping. No mass provider load or memory-exhaustion test was launched.
3. **P2 — Last valid report eviction:** quota-driven global DELETE did not exclude the refreshed scope's valid report. It could delete that report before connection validation or completion of its replacement. Reproduced with current SQL, in-memory SQLite, and 200 reports: the current scope's sole valid report disappeared. Protect it until replacement; old-cache eviction from other scopes may remain.
4. **P2 — Range overflow:** `(episode..=episode + 50)` overflowed with `episode_number=i64::MAX` and a present `episode_number_end`. An isolated probe reproduced a debug panic; release arithmetic could wrap and validate incorrectly. Use checked/saturating addition and protocol numbering limits; malformed inventory must not leave a job running indefinitely.

Independent verification at that point: 120 unit tests passed, one skipped; strict formatting/Clippy and all five frontend suites/build passed. Migration tests were not repeated in that pass to avoid overwriting the running development executable. Existing tests did not cover these four scenarios, so passing them did not close the findings.

Uppercase catalog IDs, skipped invalid entries, unique temporary files, and skipped-entry warnings were consistent with reviewed compatibility/limits. qBittorrent session-reuse implementation was unchanged by this later diff. The four issues required correction and revalidation before 1.0.4 publication. Initial password setup over the network remained out of scope.

Subsequently the user authorized publishing corrected v1.0.4. References to publication being held describe the audit sequence, not server deployment or validation of actual qBittorrent resident memory.

## User-requested follow-up fixes — originally local, unpublished

The findings below describe the PREVIOUS state. After authorization, SEC-01 through SEC-08 and additional message sanitization were corrected. Network-based initial password setup was expressly retained; loopback was not required again.

- SEC-01/02: public-download client without environment proxies or automatic redirects, destination pinned after DNS validation, IPv4-mapped normalization, and special-range rejection. Initially only the configured origin could reuse credentials or be private; subdomains did not inherit the private exception. Login limited to five same-origin hops and forms prevented cross-origin credential submission. Later allowed aliases were added with the pinned validation described above.
- SEC-03: restoration preserves current security/connection settings and `indexer_configs`. Sessions are not restored. Confirmation updated in six languages.
- SEC-04: streaming upload to private disk, one upload, maximum 512 MiB, five-minute deadline, validated SQLite header, and temporary-file cleanup on error. General JSON limit 2 MiB; credentials retain 4 KiB and inventory 128 MiB limits. Exported backups still contain secrets and need protection; no encryption using a fictitious local key was added.
- SEC-05: refresh retains three prior terminal reports per scope, favoring valid reports over errors. Admission thresholds: 200 reports and 64 MiB already stored; maximum 16 MiB per result and two active jobs. Up to two in-flight results can exceed the admission threshold: this is not an exact 64 MiB on-disk cap. Up to 128 keys per active job; terminal keys expire after seven days. The initial correction rejected new work at the scope cap instead of silently deleting the latest valid report; later safe cache eviction superseded that behavior.
- SEC-06: expired login entries removed; maximum 4,096 identities, retaining increasing lockout and IPv6 /64 grouping.
- SEC-07: catalog limits of 32 MiB compressed, 128 MiB expanded, 20,000 entries, and 512 KiB per definition; YAML validation, destination-link rejection, and temporary-file replacement. One synchronization at a time.
- SEC-08: explicit System32 paths for `reg.exe`/`sc.exe`, including tray. Existing service rights/account unchanged.
- URL secrets: sanitization when recording/returning history and diagnostics, including old messages, userinfo, and known sensitive parameters.
- Clippy corrected by moving theme tests to the end.
- Least-privilege CI permissions per job; attachments excluded from Git/Docker. Unix private data uses the same path/link guard as configuration.

### qBittorrent: identified cause and correction

Oberiz recreated the HTTP client/cookie for each operation. Cookie-authenticated polling could create a new WebUI session instead of reusing the previous one; even authentication bypass can create sessions when cookies are discarded. This pattern is documented in the [official issue](https://github.com/qbittorrent/qBittorrent/issues/24695).

It now retains a client/cookie per connection (LRU cache of at most eight), groups concurrent logins using a generation and mutex, renews after rejection, and waits 30 seconds after a failed login. Evicting an inactive connection attempts to close only its own session with a one-second deadline. It does not close other applications' sessions or change qBittorrent preferences/torrents.

Known torrents are queried by hash, including post-add waiting and category/tag checks. A complete list is fetched only when identity is unknown. The shared eight-second cache remains; Dashboard/Downloads polling does not overlap, and leaving Downloads aborts its request.

A simulated test verifies 25 queries with ONE login and 12 concurrent queries after expiration with ONE additional renewal. This is not a measurement of real NAS resident memory. Previously created sessions may consume memory until expiration; qBittorrent is not restarted or changed to force cleanup.

### Closure limitations at that stage

Follow-up verification: 117 backend unit tests passed, one optional test skipped; two migration tests passed; strict Clippy passed. Five frontend checks (languages, filters, storage, imports, reports) and build passed. Isolated local 1.0.4 startup reported health/database `ok`; `/api/settings` returned 401 without a session. Preview updated on port 2033. This was not real-server deployment or qBittorrent memory measurement.

No automatic TLS, proxy, firewall, service-account, effective NAS-permission, or GitHub-permission changes. Separate scoped keys, artifact signing, and auxiliary-tool pinning remain design improvements, not demonstrated exploited vulnerabilities. Real memory behavior requires later observation with the corrected instance. Publication was held at this historical stage.

## Original verdict and scope

Original verdict: hold publication. Two high-priority outbound-connection findings and six medium risks were identified. At that point, no corrections, commits, tags, packages, or releases had been published.

Local Oberiz code review covered authentication, internal/public routes, Cinetta inventory/reports, qBittorrent, files, backups, permissions, dependencies, UI, and distribution. Included automated tests and an isolated probe of the real IP-filter function. This is not certification of vulnerability absence or an infrastructure penetration test.

No attacks on real services or operations against real Cinetta, NAS, or qBittorrent. No passwords/private database contents inspected. Effective installation ACLs, firewall, proxy, TLS, NAS, published Docker images, actual GitHub permissions, and remote artifacts were not verified. Existing pre-audit 1.0.4 preparation changes were preserved.

## Original tests performed

- Backend: `cargo test --manifest-path backend/Cargo.toml --locked --bin oberiz`: **108 passed, 0 failed, 1 skipped**. Repeated outside network isolation because the first attempt blocked loopback fixtures and temporary-folder operations. The local instance was not stopped.
- Frontend: `test:i18n`, `test:filters`, `test:storage`, `test:imports`, `test:missing`, and build passed.
- Dependencies: OSV queried for **252 active Rust versions**, no advisories; `npm audit`, no known vulnerabilities. Does not cover OS packages, final images, CI-downloaded tools, or logic flaws.
- Rust formatting passed.
- Clippy with all targets/features and warnings as errors **failed** on `items_after_test_module` in `backend/src/db.rs:120`.
- Limited private-key/GitHub-token/AWS-key pattern scan across 750 tracked files: no matches. Not a universal secret scanner or full-history review. Untracked local attachments must not be published.
- Migration integration tests not repeated in that pass: building all binaries might overwrite the running local executable.

## High-priority findings

### SEC-01 — Automatic redirects bypass destination checks

Evidence: `backend/src/cardigann.rs:1026`, `authorize_fetch`, and `follow_redirect_once` at line 1041.

For an unrelated host, a DNS-pinned client was created without `redirect(Policy::none())`. Reqwest 0.12.28 follows redirects by default. A public server referenced by an indexer result could redirect to an internal service. That connection occurs before the application receives the response and revalidates the destination. Initial DNS pinning does not protect the next host. Impact depends on reachable internal services and their GET behavior; remote writing/execution was not demonstrated and real infrastructure was not tested.

Correction: disable automatic redirects in ALL clients on this path; validate/pin each permitted hop. Test simulated public server → loopback redirect without reaching the destination. Account for environment proxies: policy must remain valid if proxies are used.

Default behavior reference: [Reqwest ClientBuilder redirect](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.redirect).

### SEC-02 — IPv4 represented as IPv6 bypasses public-IP filtering

Evidence: `backend/src/cardigann.rs:970`, `is_public_ip`.

The extracted current function was compiled in a temporary executable:

```text
127.0.0.1                 allowed=false
::1                       allowed=false
::ffff:127.0.0.1           allowed=true
::ffff:192.168.1.1         allowed=true
```

The last two were accepted despite representing local/private IPv4 destinations. This reproduces classification failure; successful connection depends on platform and IPv6/IPv4 connectivity. A complete policy for other special ranges was also missing.

Correction: normalize IPv4-mapped addresses and apply IPv4 filtering; reject relevant nonglobal ranges with tests. Review the related-host exception in `authorize_fetch`, which returned before DNS checks. Trusting a tracker subdomain does not prove its IP is public. Keep an explicit configured-local-indexer exception, not implicit authorization for any remote-indexer subdomain.

## Medium-priority findings

### SEC-03 — Backups restore old passwords and keys

Evidence: `backend/src/backups.rs:338-352`.

`auth_sessions` was correctly excluded, but all `settings` were restored: the old password hash, integration keys, and other credentials returned. A backup predating password setup could reopen remote initial setup. This requires administrator restoration and is not direct anonymous access. Backups also contain plaintext third-party credentials and must be treated as secrets.

Proposed correction: preserve current security identity by default, obtain separate consent to restore it, verify the current password for restoration, and ensure remote bootstrap is not opened accidentally. Do not silently remove approved remote initial setup.

### SEC-04 — In-memory backup uploads without concurrency limits

Evidence: `backend/src/routes/settings.rs:33`, `backups.rs:197`, and the global body limit in `main.rs`.

Upload accepted 1 GiB and extracted complete `Bytes`; multiple administrator requests could hold several bodies in RAM. The general 512 MiB limit was too broad for most forms. Authentication reduces exposure but does not bound consumption by a valid or defective client.

Correction: small endpoint-specific limits except backups, streaming uploads with size/quota limits, and a semaphore acquired before reading bodies. Apply capacity/read deadlines to expensive internal requests.

### SEC-05 — Reports and idempotency grow without retention

Evidence: `backend/src/missing.rs:195-239`, migration 0023.

Two concurrent jobs and per-report limits were useful, but no global quota or age cleanup existed. Each completed refresh could save another large report. Even while reusing an active job, distinct idempotency keys added records. An API-key holder with the additional permission enabled could grow the database/backups.

Correction: per-scope retention, global quota, per-job action cap, and documented idempotency-key expiration. Never delete the latest valid report merely because its replacement fails.

### SEC-06 — Login-attempt entries do not expire

Evidence: `backend/src/auth.rs:218-248`.

Per-IP waiting and an Argon2 semaphore existed. Failed identities remained in the HashMap until a successful login for that identity. Reset time reset counters but did not remove entries. Many-origin attacks could grow memory throughout process lifetime. Forwarded identities behind a trusted proxy also need validation.

Correction: bounded cache with TTL/cleanup; global authentication-request limits before expensive work is queued. Preserve IPv6 /64 grouping.

### SEC-07 — No indexer-catalog download/extraction quotas

Evidence: `backend/src/indexers.rs:483` and `extract_v11` at line 1082.

ZIP download used `bytes()`; entries used `read_to_end` without per-file, cumulative, or count limits. Fixed HTTPS upstream and administrator action were required; this was not anonymous ZIP upload. A compromised upstream or uncontrolled growth could exhaust RAM/disk. Using basenames avoids classic traversal but not quotas or preexisting links.

Correction: bounded streaming, compressed/decompressed limits, YAML validation before installation, link rejection, and atomic temporary writes. Test oversized files and high-expansion ZIPs.

### SEC-08 — Windows tools resolved by name

Evidence: `backend/src/service.rs:210` (`reg`) and line 302 (`sc`).

Service installation/configuration executed tools without absolute paths. If another user controls an executable-search directory, an impostor binary could run with installer permissions. Exploitation requires local control and triggering that workflow; no actual elevation was demonstrated on this machine.

Correction: absolute System32 paths, following existing PowerShell/icacls storage-protection patterns. Also review tray/scripts. Granting all authenticated users start/stop rights is a separate availability decision better limited to the appropriate group.

## Deployment risks and additional hardening

- **HTTP/cookies:** HTTP server and default 0.0.0.0 binding; `Secure` only with `OBERIZ_COOKIE_SECURE`. Outside loopback/trusted networks, use HTTPS and Secure cookies. Do not blindly enable Secure over HTTP.
- **First administrator:** any reachable client can set the first password while absent. This is approved behavior, not a regression to fix without permission. Prepare the instance before exposure; an installation token could improve security while retaining remote setup.
- **API permissions:** one public key permits different operations; enabling reports does not make the key read-only. Consider per-integration/scoped keys, rotation, and limits.
- **Errors:** review `cardigann.rs:903`, which retained an HTTP error without common sanitization, and diagnostics displaying error history. URL query credentials could enter support reports.
- **Storage:** outbound credentials in unencrypted SQLite. Windows ACLs/private directories are the main barrier. Prefer a dedicated service identity; LocalService is shared with other Windows services. Unix data protection used chmod rather than the configuration guard's link/broad-path checks; review consistency.
- **CI/supply chain:** pinned Actions/Docker bases, nonroot Docker, prepublication tests, and checksums are positive. `cargo install cargo-cyclonedx` and packaging tools were not version-pinned. Reduce token permissions per job, pin tools, and consider artifact signing.
- **Docker context:** exclude remote attachments, secrets, backups, and alternative SQLite files; review exclusions before building/publishing.
- **Sessions/CSRF:** HttpOnly/SameSite Strict and no internal CORS are useful. No explicit Origin/Host validation was observed; test same-site/different-port and DNS-rebinding scenarios before claiming full coverage. No demonstrated CSRF bypass is asserted.

## Verified protections

- Random-salt Argon2 passwords, random opaque sessions stored as hashes, expiration/revocation on password change, and current-password verification. Credentials body limited to 4 KiB.
- Session-protected internal API; key-protected external API before JSON. External reports additionally require explicit opt-in permission.
- Parameterized reviewed queries and quoted dynamic restoration identifiers. No exploitable SQL injection identified.
- No innerHTML/dangerouslySetInnerHTML/eval usage found in reviewed React UI; CSP, nosniff, and framing protection present.
- Reports use clients without redirects, deadlines, bounded responses, stable paginated inventory, and per-job limits. GET creates no jobs.
- Reports invoke neither file operations nor qBittorrent. Local scanning is separate from operations, with no direct NAS moves.
- Manual qBittorrent deletion reflects explicit file choice; automation retains deletion blocking. Renaming is opt-in with relative-path validation, collision checks, and stopped recovery. An inherent check/rename race against external writers remains: do not promise atomicity absent from the remote API.

## Additional publication blocker (historical)

Clippy failed at `backend/src/db.rs:120` because functions followed the theme-test module. Not a runtime vulnerability, but the workflow requires warning-free Clippy. Move the module to the end and reverify without suppressing the warning.

## Original proposed order, awaiting implementation authorization

1. SEC-01/02 and missing HTTP/IP regression tests.
2. SEC-03/04/05: restore security and persistent/RAM resource quotas.
3. SEC-06/07/08: bounds, extraction, and Windows tools.
4. Agreed deployment hardening, without silent functional changes.
5. Fix Clippy, repeat tests, and review the final changes before publishing.
