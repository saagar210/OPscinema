use crate::storage::repo_models;
use crate::storage::DbConn;
use opscinema_types::{ModelRoles, ModelRolesUpdate};

pub fn get(conn: &DbConn) -> anyhow::Result<ModelRoles> {
    repo_models::get_roles(conn)
}

pub fn set(conn: &DbConn, update: &ModelRolesUpdate) -> anyhow::Result<ModelRoles> {
    repo_models::set_roles(conn, update)
}
