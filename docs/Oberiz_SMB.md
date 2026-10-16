# Oberiz server SMB connections

Historical v1.0.0/v1.0.1 document. Oberiz-managed SMB connections and credentials have been removed from the new workflow. See [Simplified storage](Oberiz_Almacenamiento_Simplificado.md).

Included in v1.0.0 for native Windows installations. Independent of NAS brand, Cinetta, or network drives connected in another Windows session.

## Optional use and compatibility

Without added SMB connections, Oberiz does not request NAS credentials, create connections, or change existing accounts, permissions, or sessions. It continues using local paths, UNC paths, and mounts already accessible to its process. A drive visible to a user but not LocalSystem remains inaccessible to the service until appropriate access is configured.

The picker opens the disks/shares view rather than systemprofile, even without SMB configuration. It does not move/delete files or modify import destinations, qBittorrent categories, or saved mappings. Publishing a version does not automatically update existing installations.

The read-only account requirement was removed at the user's request. The configured account's actual permissions apply. Scanning still does not write library files; moving/importing requires corresponding source and destination permissions.

In Settings → Library folders → SMB / CIFS servers, enter the server name/IP and NAS username/password, then select **Connect and save**. Use HTTPS when configuring from another computer. The password is never returned through the API. Windows DPAPI encrypts it for this machine so the service account can restore it. Databases and backups remain private. Credentials must be entered again after moving configuration to another Windows machine.

Windows connects without assigning drive letters and enumerates disk shares. If enumeration is blocked, supply a known share name; added shares remain listed. Incompatible credentials produce an explicit error: Oberiz does not disconnect other Windows sessions to resolve it.

The picker opens “All disks and shares”: local drives and saved SMB shares. It supports browsing UNC paths, selecting several folders, and preserving saved locations while unavailable; attempting to open them shows the error. Only NAS disks exported through SMB appear, not physical devices the NAS does not share. `OBERIZ_FS_ALLOWED_ROOTS` restrictions are respected.

At application/service startup, saved connections are restored in that process's Windows context. An offline NAS does not block startup indefinitely: restoration continues in the background after ten seconds. Browsing, scanning, and importing use UNC paths and that SMB account's permissions. NAS permissions are not changed, legacy SMB1/CIFS is not enabled, and Windows SMB client negotiation is used.

“Forget saved connection” removes credentials and shares from Oberiz configuration without forcibly disconnecting existing Windows sessions. Path mappings remain necessary when qBittorrent reports different paths, such as Linux paths.

Linux/Docker uses OS-mounted SMB shares accessible inside the container. This implementation's native credentials form does not mount Linux shares or fall back to plaintext passwords.

Verified: frontend compilation, six languages, server validation, DPAPI encryption and corruption detection, API responses without passwords/ciphertext, saved-share listing without NAS access, and credential removal. Actual NAS authentication from the service requires entering credentials in the section; credentials were not recovered from other applications or configured automatically in the existing installation.
