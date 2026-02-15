use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Storage {
    pub db_path: PathBuf,
    pub assets_root: PathBuf,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(db_path: P, assets_root: P) -> rusqlite::Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        let assets_root = assets_root.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::create_dir_all(&assets_root).ok();

        let conn = Connection::open(&db_path)?;
        Self::migrate(&conn)?;
        Ok(Self {
            db_path,
            assets_root,
        })
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let tmp = tempfile_dir();
        let db_path = tmp.join("state.sqlite");
        let assets_root = tmp.join("assets");
        Self::open(&db_path, &assets_root)
    }

    pub fn conn(&self) -> rusqlite::Result<Connection> {
        Connection::open(&self.db_path)
    }

    fn migrate(conn: &Connection) -> rusqlite::Result<()> {
        let sql = include_str!("schema/0001_init.sql");
        conn.execute_batch(sql)
    }
}

fn tempfile_dir() -> PathBuf {
    let base = std::env::temp_dir().join(format!("opscinema-{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&base);
    base
}
