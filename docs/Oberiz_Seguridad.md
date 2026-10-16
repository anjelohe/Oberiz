# Security and deployment

## Initial password

The initial password can be configured from another device's browser, including through a proxy, without accessing the server desktop. Loopback is not required. Complete setup on a trusted network before exposing the instance: while no password exists, anyone reaching the form can set it. Changes are serialized; after configuration, replacement requires the current password. Length limits, secure hashing, and session revocation on password changes remain in place.

## Private directories

Data, backups, and configuration require access restricted to the owner, administrators, and service account. Windows applies ACLs to existing and future files; Linux uses 0700 directories. The helper must target only Oberiz private directories, never a disk root, home directory, operating system directory, link, or media library.

The existing ProgramData installation requires administrator elevation to update permissions on service-created files. Previous ACLs were saved locally in `backend/backups/security-acl-20261014`. The prepared helper is `backend/backups/complete-private-acl.ps1`; run it with Windows PowerShell as administrator if the requested prompt does not appear. Its result is stored in `elevated-result.txt`. Verify DB, WAL, SHM, backup, and configuration ACLs before considering the correction complete.

Verification on October 14, 2026: helper reported success; no Users, Everyone, or Authenticated Users access rules remained in recursively inspected private files/directories. Library permissions were not changed.

## Account and network

New Windows services use LocalService rather than LocalSystem. An upgrade preserves the existing account to avoid breaking paths that depend on its permissions. Review required directory access before changing that account. Historically, SMB used explicitly configured credentials; Oberiz-managed SMB was removed in v1.0.2.

`OBERIZ_BIND_ADDRESS=127.0.0.1:2032` limits listening to loopback when using a local proxy. The default retains network listening for compatibility. Remote access requires HTTPS, `OBERIZ_COOKIE_SECURE=true`, a firewall, and `OBERIZ_TRUSTED_PROXIES` containing only actual proxy addresses. These options do not install certificates or modify the firewall automatically.

## API and files

The key is checked before deserializing bodies, including when the API is disabled. The integration has a concurrency limit. The existing key contract is preserved for Cinetta/Overseerr; the key was not automatically changed or replaced. Keep it secret; there are not yet separate per-client/scoped keys.

Automatic policies and release rejection preserve files. Manual deletion in Downloads requires confirmation and offers “Also delete downloaded files,” unchecked by default. If selected, qBittorrent deletes the files even without Oberiz NAS access or when using Cinetta inventory. A Cinetta report updates only Oberiz data. Do not interpret `superseded` as a deletion instruction without independent validation.

## Credentials and backups

Connection credentials are reversible because Oberiz must authenticate with them. Databases and backups are confidential and protected by ACLs/permissions; no full-database encryption with a new recovery key has been added. Do not publish backups or store them in shared folders. Old backups and exports outside protected directories need protection at their destination.

Administrator passwords and API keys are hashed. Historical SMB storage used machine DPAPI, which also relies on ACLs to isolate local users.

## Dependencies

Run `pwsh -File scripts/audit-dependencies.ps1` from the source checkout. It queries OSV for active Cargo versions, excluding unused optional entries, and npm for the frontend. This check is also part of CI and releases. A clean result is not a guarantee against unknown vulnerabilities or an audit of the NAS or Cinetta.
