use crate::util::time::now_utc_iso;
use opscinema_types::{BenchListResponse, BenchRecord, ModelProfile, ModelRoles, ModelRolesUpdate};
use rusqlite::params;
use uuid::Uuid;

pub fn insert_model(
    conn: &rusqlite::Connection,
    provider: &str,
    label: &str,
    digest: &str,
) -> anyhow::Result<ModelProfile> {
    let model_id = format!("{}:{}", provider, digest);
    conn.execute(
        "INSERT OR REPLACE INTO models(model_id, provider, label, digest, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![model_id, provider, label, digest, now_utc_iso()],
    )?;
    Ok(ModelProfile {
        model_id,
        provider: provider.to_string(),
        label: label.to_string(),
        digest: digest.to_string(),
    })
}

pub fn list_models(conn: &rusqlite::Connection) -> anyhow::Result<Vec<ModelProfile>> {
    let mut stmt = conn
        .prepare("SELECT model_id, provider, label, digest FROM models ORDER BY created_at DESC")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ModelProfile {
                model_id: r.get(0)?,
                provider: r.get(1)?,
                label: r.get(2)?,
                digest: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn remove_model(conn: &rusqlite::Connection, model_id: &str) -> anyhow::Result<bool> {
    let affected = conn.execute("DELETE FROM models WHERE model_id=?1", params![model_id])?;
    if affected > 0 {
        conn.execute(
            "UPDATE model_roles SET
             tutorial_generation = CASE WHEN tutorial_generation=?1 THEN NULL ELSE tutorial_generation END,
             screen_explainer = CASE WHEN screen_explainer=?1 THEN NULL ELSE screen_explainer END,
             anchor_grounding = CASE WHEN anchor_grounding=?1 THEN NULL ELSE anchor_grounding END
             WHERE id=1",
            params![model_id],
        )?;
    }
    Ok(affected > 0)
}

pub fn set_roles(
    conn: &rusqlite::Connection,
    update: &ModelRolesUpdate,
) -> anyhow::Result<ModelRoles> {
    conn.execute(
        "INSERT INTO model_roles(id, tutorial_generation, screen_explainer, anchor_grounding)
         VALUES (1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET tutorial_generation=excluded.tutorial_generation, screen_explainer=excluded.screen_explainer, anchor_grounding=excluded.anchor_grounding",
        params![update.tutorial_generation, update.screen_explainer, update.anchor_grounding],
    )?;
    Ok(ModelRoles {
        tutorial_generation: update.tutorial_generation.clone(),
        screen_explainer: update.screen_explainer.clone(),
        anchor_grounding: update.anchor_grounding.clone(),
    })
}

pub fn get_roles(conn: &rusqlite::Connection) -> anyhow::Result<ModelRoles> {
    let mut stmt = conn.prepare("SELECT tutorial_generation, screen_explainer, anchor_grounding FROM model_roles WHERE id=1")?;
    let roles = stmt
        .query_row([], |r| {
            Ok(ModelRoles {
                tutorial_generation: r.get(0)?,
                screen_explainer: r.get(1)?,
                anchor_grounding: r.get(2)?,
            })
        })
        .unwrap_or(ModelRoles {
            tutorial_generation: None,
            screen_explainer: None,
            anchor_grounding: None,
        });
    Ok(roles)
}

pub fn record_benchmark(
    conn: &rusqlite::Connection,
    model_id: &str,
    score: i32,
) -> anyhow::Result<BenchRecord> {
    let bench = BenchRecord {
        bench_id: Uuid::new_v4(),
        model_id: model_id.to_string(),
        score,
        created_at: chrono::Utc::now(),
    };
    conn.execute(
        "INSERT INTO benchmarks(bench_id, model_id, score, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            bench.bench_id.to_string(),
            bench.model_id,
            bench.score,
            now_utc_iso()
        ],
    )?;
    Ok(bench)
}

pub fn list_benchmarks(conn: &rusqlite::Connection) -> anyhow::Result<BenchListResponse> {
    let mut stmt = conn.prepare(
        "SELECT bench_id, model_id, score, created_at FROM benchmarks ORDER BY created_at DESC",
    )?;
    let benches = stmt
        .query_map([], |r| {
            Ok(BenchRecord {
                bench_id: Uuid::parse_str(&r.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::nil()),
                model_id: r.get(1)?,
                score: r.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&r.get::<_, String>(3)?)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BenchListResponse { benches })
}
