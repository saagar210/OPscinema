use anyhow::Context;
use std::path::{Path, PathBuf};

pub fn ensure_dir(path: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(path).with_context(|| format!("create dir {}", path.display()))?;
    Ok(())
}

pub fn write_file(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    let f = std::fs::OpenOptions::new().read(true).open(&tmp)?;
    f.sync_all()?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

pub fn list_files_sorted(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for e in walkdir::WalkDir::new(root) {
        let e = e?;
        if e.file_type().is_file() {
            files.push(e.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}
