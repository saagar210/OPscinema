use crate::api::Backend;
use crate::model_dock::{adapters, bench, registry, roles};
use crate::storage::{repo_jobs, repo_models};
use opscinema_export_manifest::ModelPin;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, BenchListRequest, BenchListResponse, BenchRunRequest,
    JobHandle, JobStatus, MlxRunRequest, ModelProfile, ModelRegisterRequest, ModelRoles,
    ModelRolesUpdate, ModelsListRequest, ModelsListResponse, ModelsRemoveRequest,
    ModelsRemoveResponse, OllamaListRequest, OllamaListResponse, OllamaPullRequest,
    OllamaRunRequest,
};
use std::collections::BTreeMap;

pub fn models_list(backend: &Backend, _req: ModelsListRequest) -> AppResult<ModelsListResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    registry::list(&conn).map_err(internal_anyhow)
}

pub fn models_register(backend: &Backend, req: ModelRegisterRequest) -> AppResult<ModelProfile> {
    let conn = backend.storage.conn().map_err(db_err)?;
    registry::register(&conn, &req.provider, &req.label, &req.digest).map_err(internal_anyhow)
}

pub fn models_remove(
    backend: &Backend,
    req: ModelsRemoveRequest,
) -> AppResult<ModelsRemoveResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let removed = repo_models::remove_model(&conn, &req.model_id).map_err(internal_anyhow)?;
    Ok(ModelsRemoveResponse { removed })
}

pub fn model_roles_get(backend: &Backend) -> AppResult<ModelRoles> {
    let conn = backend.storage.conn().map_err(db_err)?;
    roles::get(&conn).map_err(internal_anyhow)
}

pub fn model_roles_set(backend: &Backend, req: ModelRolesUpdate) -> AppResult<ModelRoles> {
    let conn = backend.storage.conn().map_err(db_err)?;
    roles::set(&conn, &req).map_err(internal_anyhow)
}

pub fn ollama_list(backend: &Backend, req: OllamaListRequest) -> AppResult<OllamaListResponse> {
    let host = req.host.unwrap_or_else(ollama_default_host);
    let policy = backend
        .network_policy
        .lock()
        .map_err(|_| internal("lock poisoned"))?
        .clone();
    policy.check_host(&host)?;
    adapters::ollama::list_models(&policy, &host)
        .map(|models| OllamaListResponse { models })
        .map_err(internal_anyhow)
}

pub fn ollama_pull(backend: &Backend, req: OllamaPullRequest) -> AppResult<JobHandle> {
    let host = req.host.unwrap_or_else(ollama_default_host);
    let policy = backend
        .network_policy
        .lock()
        .map_err(|_| internal("lock poisoned"))?
        .clone();
    policy.check_host(&host)?;

    let conn = backend.storage.conn().map_err(db_err)?;
    let job_id = repo_jobs::create_job(&conn, "ollama_pull", None).map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Running, None, None);

    let pull_res = adapters::ollama::pull_model(&policy, &host, &req.model);
    match pull_res {
        Ok(_) => {
            let digest = blake3::hash(format!("{host}:{}", req.model).as_bytes())
                .to_hex()
                .to_string();
            let _ = registry::register(&conn, "ollama", &req.model, &digest);
            let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Succeeded, None, None);
            Ok(JobHandle { job_id })
        }
        Err(e) => {
            let app_err = internal(&e.to_string());
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                JobStatus::Failed,
                None,
                Some(app_err.clone()),
            );
            Err(app_err)
        }
    }
}

pub fn ollama_run(backend: &Backend, req: OllamaRunRequest) -> AppResult<JobHandle> {
    let host = ollama_default_host();
    let policy = backend
        .network_policy
        .lock()
        .map_err(|_| internal("lock poisoned"))?
        .clone();
    policy.check_host(&host)?;

    let conn = backend.storage.conn().map_err(db_err)?;
    let job_id = repo_jobs::create_job(&conn, "ollama_run", None).map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Running, None, None);

    let run_res = adapters::ollama::run_prompt(&policy, &host, &req.model_id, &req.prompt);
    match run_res {
        Ok(output) => {
            let _ = backend.assets.put(&conn, output.as_bytes(), None);
            let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Succeeded, None, None);
            Ok(JobHandle { job_id })
        }
        Err(e) => {
            let app_err = internal(&e.to_string());
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                JobStatus::Failed,
                None,
                Some(app_err.clone()),
            );
            Err(app_err)
        }
    }
}

pub fn mlx_run(backend: &Backend, req: MlxRunRequest) -> AppResult<JobHandle> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let job_id = repo_jobs::create_job(&conn, "mlx_run", None).map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Running, None, None);

    let run_res = adapters::mlx::run_local(&req.model_id, &req.prompt);
    match run_res {
        Ok(output) => {
            let digest = blake3::hash(req.model_id.as_bytes()).to_hex().to_string();
            let _ = registry::register(&conn, "mlx", &req.model_id, &digest);
            let _ = backend.assets.put(&conn, output.as_bytes(), None);
            let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Succeeded, None, None);
            Ok(JobHandle { job_id })
        }
        Err(e) => {
            let app_err = internal(&e.to_string());
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                JobStatus::Failed,
                None,
                Some(app_err.clone()),
            );
            Err(app_err)
        }
    }
}

pub fn bench_run(backend: &Backend, req: BenchRunRequest) -> AppResult<JobHandle> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let job_id = repo_jobs::create_job(&conn, "bench_run", None).map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Running, None, None);
    let score = (blake3::hash(format!("{}:{}", req.model_id, req.benchmark).as_bytes()).as_bytes()
        [0] as i32)
        .max(1);
    let run_res = bench::record(&conn, &req.model_id, score);
    match run_res {
        Ok(_) => {
            let _ = repo_jobs::update_job_status(&conn, job_id, JobStatus::Succeeded, None, None);
            Ok(JobHandle { job_id })
        }
        Err(e) => {
            let app_err = internal(&e.to_string());
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                JobStatus::Failed,
                None,
                Some(app_err.clone()),
            );
            Err(app_err)
        }
    }
}

pub fn bench_list(backend: &Backend, _req: BenchListRequest) -> AppResult<BenchListResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    bench::list(&conn).map_err(internal_anyhow)
}

pub fn collect_model_pins(backend: &Backend) -> AppResult<Vec<ModelPin>> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let roles = roles::get(&conn).map_err(internal_anyhow)?;
    let models = registry::list(&conn).map_err(internal_anyhow)?.models;
    let by_id = models
        .into_iter()
        .map(|m| (m.model_id.clone(), m))
        .collect::<BTreeMap<_, _>>();

    let mut pins = Vec::new();
    push_role_pin(
        &mut pins,
        &by_id,
        "tutorial_generation",
        roles.tutorial_generation,
    );
    push_role_pin(
        &mut pins,
        &by_id,
        "screen_explainer",
        roles.screen_explainer,
    );
    push_role_pin(
        &mut pins,
        &by_id,
        "anchor_grounding",
        roles.anchor_grounding,
    );
    pins.sort_by(|a, b| a.role.cmp(&b.role));
    Ok(pins)
}

fn push_role_pin(
    out: &mut Vec<ModelPin>,
    by_id: &BTreeMap<String, ModelProfile>,
    role: &str,
    model_id: Option<String>,
) {
    if let Some(model_id) = model_id {
        if let Some(model) = by_id.get(&model_id) {
            out.push(ModelPin {
                role: role.to_string(),
                model_id: model.model_id.clone(),
                digest: model.digest.clone(),
            });
        }
    }
}

fn ollama_default_host() -> String {
    std::env::var("OPSCINEMA_OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn db_err<E: std::fmt::Display>(e: E) -> AppError {
    AppError {
        code: AppErrorCode::Db,
        message: "database error".to_string(),
        details: Some(e.to_string()),
        recoverable: false,
        action_hint: None,
    }
}

fn internal(msg: &str) -> AppError {
    AppError {
        code: AppErrorCode::Internal,
        message: msg.to_string(),
        details: None,
        recoverable: false,
        action_hint: None,
    }
}

fn internal_anyhow(e: anyhow::Error) -> AppError {
    internal(&e.to_string())
}
