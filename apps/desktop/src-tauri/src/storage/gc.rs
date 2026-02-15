use crate::storage::asset_store::AssetStore;
use serde::Serialize;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcReport {
    pub referenced_count: usize,
    pub orphan_count: usize,
    pub deleted_count: usize,
    pub orphan_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct StorageGcRanPayload {
    dry_run: bool,
    referenced_count: usize,
    orphan_count: usize,
    deleted_count: usize,
    orphan_ids: Vec<String>,
}

pub fn collect_referenced_asset_ids(
    conn: &rusqlite::Connection,
) -> anyhow::Result<BTreeSet<String>> {
    let mut refs = BTreeSet::new();

    // assets referenced directly in event payloads
    let mut stmt = conn.prepare("SELECT payload_canon_json FROM events")?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for raw in rows {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            collect_asset_ids_from_json(&v, &mut refs);
        }
    }

    // exports table references manifest asset
    let mut stmt = conn.prepare("SELECT manifest_asset_id FROM exports")?;
    let export_ids = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    refs.extend(export_ids);

    // verifier outputs/logs references
    let mut stmt = conn.prepare("SELECT result_asset_id, logs_asset_id FROM verifier_runs")?;
    let vr_rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (result, logs) in vr_rows {
        refs.insert(result);
        if let Some(logs) = logs {
            refs.insert(logs);
        }
    }

    // snapshots may include asset IDs inside JSON
    for table in ["steps_snapshot", "anchors_snapshot"] {
        let mut stmt = conn.prepare(&format!("SELECT {} FROM {}", json_col(table), table))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for raw in rows {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                collect_asset_ids_from_json(&v, &mut refs);
            }
        }
    }

    Ok(refs)
}

pub fn gc_orphan_assets(
    conn: &rusqlite::Connection,
    store: &AssetStore,
    dry_run: bool,
) -> anyhow::Result<GcReport> {
    let referenced = collect_referenced_asset_ids(conn)?;

    let mut stmt = conn.prepare("SELECT asset_id FROM assets")?;
    let all_ids = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut orphan_ids = all_ids
        .into_iter()
        .filter(|id| !referenced.contains(id))
        .collect::<Vec<_>>();
    orphan_ids.sort();

    let mut deleted_count = 0usize;
    if !dry_run {
        for asset_id in &orphan_ids {
            let path = store.path_for(asset_id);
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            conn.execute(
                "DELETE FROM assets WHERE asset_id=?1",
                rusqlite::params![asset_id],
            )?;
            deleted_count += 1;
        }
    }

    Ok(GcReport {
        referenced_count: referenced.len(),
        orphan_count: orphan_ids.len(),
        deleted_count,
        orphan_ids,
    })
}

pub fn gc_orphan_assets_with_audit(
    conn: &mut rusqlite::Connection,
    store: &AssetStore,
    dry_run: bool,
    audit_session_id: Option<Uuid>,
) -> anyhow::Result<GcReport> {
    let report = gc_orphan_assets(conn, store, dry_run)?;
    if !dry_run {
        if let Some(session_id) = audit_session_id {
            let payload = StorageGcRanPayload {
                dry_run,
                referenced_count: report.referenced_count,
                orphan_count: report.orphan_count,
                deleted_count: report.deleted_count,
                orphan_ids: report.orphan_ids.clone(),
            };
            let _ = crate::storage::event_store::append_event(
                conn,
                session_id,
                "StorageGcRan",
                &payload,
                None,
            )?;
        }
    }
    Ok(report)
}

fn json_col(table: &str) -> &'static str {
    match table {
        "steps_snapshot" => "steps_json",
        "anchors_snapshot" => "anchors_json",
        _ => "",
    }
}

fn collect_asset_ids_from_json(v: &serde_json::Value, refs: &mut BTreeSet<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                let key = k.to_lowercase();
                if (key == "asset_id" || key.ends_with("_asset_id")) && val.is_string() {
                    if let Some(id) = val.as_str() {
                        refs.insert(id.to_string());
                    }
                }
                collect_asset_ids_from_json(val, refs);
            }
        }
        serde_json::Value::Array(arr) => {
            for it in arr {
                collect_asset_ids_from_json(it, refs);
            }
        }
        _ => {}
    }
}
