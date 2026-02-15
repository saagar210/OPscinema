use crate::util::hash::blake3_hex;
use crate::util::time::now_utc_iso;
use rusqlite::params;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AssetStore {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub enum CrashPoint {
    AfterAssetWriteBeforeDb,
}

impl AssetStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn put(
        &self,
        conn: &rusqlite::Connection,
        bytes: &[u8],
        crash: Option<CrashPoint>,
    ) -> anyhow::Result<String> {
        let asset_id = blake3_hex(bytes);
        let rel = format!("{}/{}/{}", &asset_id[0..2], &asset_id[2..4], asset_id);
        let final_path = self.root.join(&rel);
        if !final_path.exists() {
            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let tmp_path = final_path.with_extension("tmp");
            std::fs::write(&tmp_path, bytes)?;
            let file = std::fs::OpenOptions::new().read(true).open(&tmp_path)?;
            file.sync_all()?;
            std::fs::rename(&tmp_path, &final_path)?;
        }

        if matches!(crash, Some(CrashPoint::AfterAssetWriteBeforeDb)) {
            anyhow::bail!("simulated crash after asset write")
        }

        conn.execute(
            "INSERT OR IGNORE INTO assets(asset_id, rel_path, size_bytes, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![asset_id, rel, bytes.len() as i64, now_utc_iso()],
        )?;
        Ok(asset_id)
    }

    pub fn path_for(&self, asset_id: &str) -> PathBuf {
        self.root.join(format!(
            "{}/{}/{}",
            &asset_id[0..2],
            &asset_id[2..4],
            asset_id
        ))
    }
}
