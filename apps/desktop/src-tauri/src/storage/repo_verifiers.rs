use opscinema_types::{AssetRef, VerifierListResponse, VerifierResultDetail, VerifierSpec};
use rusqlite::{params, OptionalExtension};

pub fn seed_default_verifiers(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let defaults = vec![
        VerifierSpec {
            verifier_id: "shell.safe_echo".to_string(),
            kind: "shell".to_string(),
            timeout_secs: 5,
            command_allowlist: vec!["echo".to_string()],
        },
        VerifierSpec {
            verifier_id: "file.exists".to_string(),
            kind: "file".to_string(),
            timeout_secs: 5,
            command_allowlist: vec![],
        },
    ];

    for spec in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO verifiers(verifier_id, kind, spec_json, enabled) VALUES (?1, ?2, ?3, 1)",
            params![spec.verifier_id, spec.kind, serde_json::to_string(&spec)?],
        )?;
    }
    Ok(())
}

pub fn list_verifiers(
    conn: &rusqlite::Connection,
    include_disabled: bool,
) -> anyhow::Result<VerifierListResponse> {
    let sql = if include_disabled {
        "SELECT spec_json FROM verifiers"
    } else {
        "SELECT spec_json FROM verifiers WHERE enabled=1"
    };
    let mut stmt = conn.prepare(sql)?;
    let verifiers = stmt
        .query_map([], |r| {
            let raw: String = r.get(0)?;
            let spec: VerifierSpec = serde_json::from_str(&raw).unwrap_or(VerifierSpec {
                verifier_id: "invalid".to_string(),
                kind: "invalid".to_string(),
                timeout_secs: 1,
                command_allowlist: vec![],
            });
            Ok(spec)
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VerifierListResponse { verifiers })
}

pub fn insert_run(
    conn: &rusqlite::Connection,
    run_id: uuid::Uuid,
    verifier_id: &str,
    session_id: uuid::Uuid,
    status: &str,
    result_asset_id: &str,
    logs_asset_id: Option<&str>,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO verifier_runs(run_id, verifier_id, session_id, status, result_asset_id, logs_asset_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            run_id.to_string(),
            verifier_id,
            session_id.to_string(),
            status,
            result_asset_id,
            logs_asset_id,
            crate::util::time::now_utc_iso()
        ],
    )?;
    Ok(())
}

pub fn get_run(
    conn: &rusqlite::Connection,
    run_id: uuid::Uuid,
) -> anyhow::Result<Option<VerifierResultDetail>> {
    let row = conn
        .query_row(
            "SELECT run_id, verifier_id, status, result_asset_id, logs_asset_id FROM verifier_runs WHERE run_id=?1",
            params![run_id.to_string()],
            |r| {
                let parsed_run_id = uuid::Uuid::parse_str(&r.get::<_, String>(0)?)
                    .unwrap_or_else(|_| uuid::Uuid::nil());
                let logs_asset_id: Option<String> = r.get(4)?;
                Ok(VerifierResultDetail {
                    run_id: parsed_run_id,
                    verifier_id: r.get(1)?,
                    status: r.get(2)?,
                    result_asset: AssetRef {
                        asset_id: r.get(3)?,
                    },
                    logs_asset: logs_asset_id.map(|asset_id| AssetRef { asset_id }),
                })
            },
        )
        .optional()?;
    Ok(row)
}
