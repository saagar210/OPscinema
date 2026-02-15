use crate::storage::repo_models;
use crate::storage::DbConn;
use opscinema_types::{ModelProfile, ModelsListResponse};

pub fn register(
    conn: &DbConn,
    provider: &str,
    label: &str,
    digest: &str,
) -> anyhow::Result<ModelProfile> {
    repo_models::insert_model(conn, provider, label, digest)
}

pub fn list(conn: &DbConn) -> anyhow::Result<ModelsListResponse> {
    Ok(ModelsListResponse {
        models: repo_models::list_models(conn)?,
    })
}
