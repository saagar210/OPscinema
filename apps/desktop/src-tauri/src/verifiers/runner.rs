use crate::storage::asset_store::AssetStore;
use crate::storage::repo_verifiers;
use crate::storage::DbConn;
use opscinema_types::{AssetRef, VerifierResultDetail};
use uuid::Uuid;

pub fn persist_result(
    conn: &DbConn,
    store: &AssetStore,
    session_id: Uuid,
    verifier_id: &str,
    status: &str,
    output: &str,
    logs: Option<&str>,
) -> anyhow::Result<VerifierResultDetail> {
    let run_id = Uuid::new_v4();
    let result_asset_id = store.put(conn, output.as_bytes(), None)?;
    let logs_asset_id = logs
        .map(|l| store.put(conn, l.as_bytes(), None))
        .transpose()?;
    repo_verifiers::insert_run(
        conn,
        run_id,
        verifier_id,
        session_id,
        status,
        &result_asset_id,
        logs_asset_id.as_deref(),
    )?;

    Ok(VerifierResultDetail {
        run_id,
        verifier_id: verifier_id.to_string(),
        status: status.to_string(),
        result_asset: AssetRef {
            asset_id: result_asset_id,
        },
        logs_asset: logs_asset_id.map(|asset_id| AssetRef { asset_id }),
    })
}
