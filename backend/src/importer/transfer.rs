//! Moving bytes from a completed download into the library: hardlink, copy,
//! move (with a cross-device fallback), and finding which files in a torrent
//! are actually media worth importing.
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Whether two files hold exactly the same bytes. Equal size alone is not
/// enough to call two files "the same" — two different releases can happen
/// to land on an equal byte count — so this always reads and compares the
/// actual content, short-circuiting on the first mismatch.
pub(super) fn files_have_identical_content(a: &Path, b: &Path) -> std::io::Result<bool> {
    if fs::metadata(a)?.len() != fs::metadata(b)?.len() {
        return Ok(false);
    }
    let mut fa = fs::File::open(a)?;
    let mut fb = fs::File::open(b)?;
    let mut buf_a = [0u8; 64 * 1024];
    let mut buf_b = [0u8; 64 * 1024];
    loop {
        let na = fa.read(&mut buf_a)?;
        let nb = fb.read(&mut buf_b)?;
        if na != nb || buf_a[..na] != buf_b[..nb] {
            return Ok(false);
        }
        if na == 0 {
            return Ok(true);
        }
    }
}

pub(super) fn transfer_file(src: &Path, dst: &Path, method: &str) -> std::io::Result<()> {
    match method {
        "hardlink" => fs::hard_link(src, dst),
        "copy" => copy_atomic(src, dst),
        "move" => move_file(src, dst),
        _ => match fs::hard_link(src, dst) {
            Ok(()) => Ok(()),
            Err(_) => copy_atomic(src, dst),
        },
    }
}

/// `fs::rename` fails whenever source and destination are on different
/// filesystems or volumes — a very common setup (downloads on one disk,
/// the library on another). Fall back to copy + size verification + delete,
/// and never remove the source until the copy is confirmed intact.
fn move_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => copy_then_remove_source(src, dst),
    }
}

/// The cross-device fallback, split out so it can be exercised directly by
/// tests without needing to force a genuine cross-filesystem rename failure.
fn copy_then_remove_source(src: &Path, dst: &Path) -> std::io::Result<()> {
    copy_atomic(src, dst)?;
    fs::remove_file(src)
}

/// Copies through a temporary sibling file and renames it into place only
/// once the copy is verified complete (same size as the source). Copying
/// straight to `dst` meant a crash or power loss mid-copy left a truncated
/// file sitting at the real library path, indistinguishable from a finished
/// import; the temp file this leaves behind instead is obviously incomplete
/// and never on the path anything else reads media from.
fn copy_atomic(src: &Path, dst: &Path) -> std::io::Result<()> {
    let temp = temp_sibling(dst);
    let result = (|| {
        let copied_bytes = fs::copy(src, &temp)?;
        let source_bytes = fs::metadata(src)?.len();
        if copied_bytes != source_bytes {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("copied {copied_bytes} bytes but source is {source_bytes} bytes"),
            ));
        }
        fs::rename(&temp, dst)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn temp_sibling(dst: &Path) -> PathBuf {
    let mut name = dst.file_name().unwrap_or_default().to_os_string();
    name.push(".oberiz-tmp");
    dst.with_file_name(name)
}

pub(super) fn collect_media_files(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    const MEDIA_EXTS: &[&str] = &[
        "mkv", "mp4", "avi", "m4v", "mov", "ts", "m2ts", "wmv", "webm",
    ];
    if path.is_file() {
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if MEDIA_EXTS.contains(&ext.as_str()) {
            out.push(path.to_path_buf());
        }
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        // Don't follow directory symlinks while walking a torrent's content
        // path: a link back to an ancestor, or a cycle between two
        // directories, would otherwise recurse until the stack or the
        // process's open-file limit gives out.
        if entry.file_type()?.is_symlink() {
            continue;
        }
        let p = entry.path();
        if p.is_dir() {
            collect_media_files(&p, out)?;
        } else {
            let name = p
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if name.contains("sample") {
                continue;
            }
            let ext = p
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if MEDIA_EXTS.contains(&ext.as_str()) {
                out.push(p);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "oberiz-importer-test-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn files_have_identical_content_rejects_equal_size_different_bytes() {
        let root = temp_dir("identical-size-mismatch");
        let a = root.join("a.mkv");
        let b = root.join("b.mkv");
        fs::write(&a, b"NEW!").unwrap();
        fs::write(&b, b"OLD!").unwrap();

        assert!(!files_have_identical_content(&a, &b).unwrap());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn files_have_identical_content_accepts_genuinely_equal_files() {
        let root = temp_dir("identical-match");
        let a = root.join("a.mkv");
        let b = root.join("b.mkv");
        fs::write(&a, b"same payload bytes").unwrap();
        fs::write(&b, b"same payload bytes").unwrap();

        assert!(files_have_identical_content(&a, &b).unwrap());

        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn collect_media_files_does_not_follow_a_symlink_cycle() {
        let root = temp_dir("symlink-cycle");
        fs::create_dir_all(root.join("real")).unwrap();
        fs::write(root.join("real").join("episode.mkv"), b"video").unwrap();
        // A directory symlink pointing back at its own ancestor: following it
        // would recurse forever.
        std::os::unix::fs::symlink(&root, root.join("real").join("loop")).unwrap();

        let mut found = Vec::new();
        collect_media_files(&root, &mut found).unwrap();

        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("episode.mkv"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn collect_media_files_skips_samples_and_non_media_extensions() {
        let root = temp_dir("collect");
        fs::create_dir_all(root.join("Season 01")).unwrap();
        fs::write(root.join("Season 01").join("episode.mkv"), b"video").unwrap();
        fs::write(root.join("Season 01").join("episode-sample.mkv"), b"x").unwrap();
        fs::write(root.join("readme.txt"), b"not media").unwrap();

        let mut found = Vec::new();
        collect_media_files(&root, &mut found).unwrap();

        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("episode.mkv"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn transfer_file_copy_leaves_source_intact() {
        let root = temp_dir("copy");
        let src = root.join("source.mkv");
        let dst = root.join("dest.mkv");
        fs::write(&src, b"payload").unwrap();

        transfer_file(&src, &dst, "copy").unwrap();

        assert!(src.exists(), "copy must not remove the source");
        assert_eq!(fs::read(&dst).unwrap(), b"payload");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn transfer_file_move_removes_source_on_the_same_filesystem() {
        let root = temp_dir("move");
        let src = root.join("source.mkv");
        let dst = root.join("dest.mkv");
        fs::write(&src, b"payload").unwrap();

        transfer_file(&src, &dst, "move").unwrap();

        assert!(!src.exists(), "move must remove the source once done");
        assert_eq!(fs::read(&dst).unwrap(), b"payload");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cross_device_move_fallback_copies_then_removes_the_source() {
        // Exercises the exact fallback `move_file` uses when `fs::rename`
        // fails because source and destination are on different volumes —
        // something that can't be forced portably inside a single temp dir,
        // so the fallback itself is tested directly instead.
        let root = temp_dir("fallback");
        let src = root.join("source.mkv");
        let dst = root.join("dest.mkv");
        fs::write(&src, b"payload").unwrap();

        copy_then_remove_source(&src, &dst).unwrap();

        assert!(
            !src.exists(),
            "source must be removed once the copy is verified"
        );
        assert_eq!(fs::read(&dst).unwrap(), b"payload");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn transfer_file_hardlink_shares_content_with_the_source() {
        let root = temp_dir("hardlink");
        let src = root.join("source.mkv");
        let dst = root.join("dest.mkv");
        fs::write(&src, b"payload").unwrap();

        transfer_file(&src, &dst, "hardlink").unwrap();

        assert!(src.exists(), "hardlink must keep the source too");
        assert_eq!(fs::read(&dst).unwrap(), b"payload");

        let _ = fs::remove_dir_all(&root);
    }
}
