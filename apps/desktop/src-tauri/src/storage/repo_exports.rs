use crate::util::time::now_utc_iso;
use opscinema_types::{ExportResult, ExportWarning};
use rusqlite::params;
use uuid::Uuid;

pub fn insert_export(
    conn: &rusqlite::Connection,
    session_id: Uuid,
    bundle_type: &str,
    output_path: &str,
    manifest_asset_id: &str,
    bundle_hash: &str,
    warnings: &[ExportWarning],
) -> anyhow::Result<ExportResult> {
    let export_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO exports(export_id, session_id, bundle_type, output_path, manifest_asset_id, bundle_hash, warnings_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            export_id.to_string(),
            session_id.to_string(),
            bundle_type,
            output_path,
            manifest_asset_id,
            bundle_hash,
            serde_json::to_string(warnings)?,
            now_utc_iso(),
        ],
    )?;

    Ok(ExportResult {
        export_id,
        output_path: output_path.to_string(),
        bundle_hash: bundle_hash.to_string(),
        warnings: warnings.to_vec(),
    })
}

pub fn list_exports(
    conn: &rusqlite::Connection,
    session_id: Option<Uuid>,
) -> anyhow::Result<Vec<ExportResult>> {
    let mut out = Vec::new();
    if let Some(session_id) = session_id {
        let mut stmt = conn.prepare(
            "SELECT export_id, output_path, bundle_hash, warnings_json
             FROM exports
             WHERE session_id=?1
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![session_id.to_string()], |r| {
            let warnings_json: String = r.get(3)?;
            Ok(ExportResult {
                export_id: Uuid::parse_str(&r.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::nil()),
                output_path: r.get(1)?,
                bundle_hash: r.get(2)?,
                warnings: serde_json::from_str(&warnings_json).unwrap_or_default(),
            })
        })?;
        for row in rows {
            out.push(row?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT export_id, output_path, bundle_hash, warnings_json
             FROM exports
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            let warnings_json: String = r.get(3)?;
            Ok(ExportResult {
                export_id: Uuid::parse_str(&r.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::nil()),
                output_path: r.get(1)?,
                bundle_hash: r.get(2)?,
                warnings: serde_json::from_str(&warnings_json).unwrap_or_default(),
            })
        })?;
        for row in rows {
            out.push(row?);
        }
    }
    Ok(out)
}
