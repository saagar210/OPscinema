use crate::storage::repo_models;
use crate::storage::DbConn;
use opscinema_types::{BenchListResponse, BenchRecord};

pub fn record(conn: &DbConn, model_id: &str, score: i32) -> anyhow::Result<BenchRecord> {
    repo_models::record_benchmark(conn, model_id, score)
}

pub fn list(conn: &DbConn) -> anyhow::Result<BenchListResponse> {
    repo_models::list_benchmarks(conn)
}
