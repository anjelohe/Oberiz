//! Private application storage only; never changes media/NAS permissions.
pub(crate) fn protect_directory(path: &std::path::Path) -> anyhow::Result<()> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let normalized = absolute
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase();
    let broad = [
        "",
        "/",
        "/etc",
        "/usr",
        "/var",
        "/var/lib",
        "/home",
        "/root",
        "/tmp",
        "c:/users",
        "c:/programdata",
        "c:/program files",
        "c:/program files (x86)",
    ];
    if absolute.parent().is_none()
        || broad.contains(&normalized.as_str())
        || absolute == std::env::current_dir()?
        || ["USERPROFILE", "HOME"]
            .iter()
            .filter_map(std::env::var_os)
            .any(|home| absolute.as_os_str() == home)
        || (cfg!(windows) && (normalized.starts_with("//") || normalized.starts_with("c:/windows")))
        || absolute
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        anyhow::bail!(
            "Refusing to change permissions on a broad, network, or system directory; configure a dedicated Oberiz private storage directory"
        );
    }
    std::fs::create_dir_all(path)?;
    if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
        anyhow::bail!("Private storage must not be a symlink");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let powershell = std::path::PathBuf::from(
            std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()),
        )
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let trusted_modules = powershell.parent().unwrap().join("Modules");
        let output = std::process::Command::new(powershell)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                include_str!("storage_acl.ps1"),
            ])
            .env("OBERIZ_ACL_TARGET", std::fs::canonicalize(path)?)
            .env("PSModulePath", trusted_modules)
            .creation_flags(0x08000000)
            .output()?;
        if !output.status.success() {
            #[cfg(test)]
            eprintln!(
                "ACL fixture helper: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            anyhow::bail!(
                "Cannot protect Oberiz private storage ACL; check ownership and permissions"
            );
        }
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn protects_private_directory_without_modifying_its_contents() {
        let directory = std::env::temp_dir().join(format!(
            "oberiz-private-acl-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let file = directory.join("synthetic.txt");
        std::fs::write(&file, b"synthetic test data").unwrap();
        super::protect_directory(&directory).unwrap();
        assert_eq!(std::fs::read(&file).unwrap(), b"synthetic test data");
        std::fs::remove_file(file).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
