use std::sync::Arc;

use tauri::State;

use crate::api::{
    agent_plant, anchors, app, capture, evidence, exports, jobs, model_dock, ocr, proof,
    runtime_events::RuntimeEventBus, sessions, slicer, steps, timeline, verifiers, Backend,
};
use opscinema_types::*;

fn backend<'a>(state: &'a State<'a, Arc<Backend>>) -> &'a Backend {
    state.inner().as_ref()
}

fn should_emit_capture_status_from_wrapper(backend: &Backend) -> bool {
    backend
        .capture_status_hook
        .lock()
        .map(|hook| hook.is_none())
        .unwrap_or(true)
}

fn emit_job_lifecycle(bus: &RuntimeEventBus, backend: &Backend, job: &JobHandle) -> AppResult<()> {
    let detail = jobs::jobs_get(backend, JobsGetRequest { job_id: job.job_id })?;
    if let Some(progress) = detail.progress.clone() {
        bus.emit_job_progress_stage(
            job.job_id,
            &progress.stage,
            progress.pct,
            progress.counters.done,
            progress.counters.total,
        )?;
    }
    let mut statuses = vec![JobStatus::Queued];
    let should_emit_running = detail.started_at.is_some()
        || matches!(
            detail.status,
            JobStatus::Running | JobStatus::Succeeded | JobStatus::Failed
        );
    if should_emit_running {
        statuses.push(JobStatus::Running);
    }
    if detail.status != JobStatus::Queued && detail.status != JobStatus::Running {
        statuses.push(detail.status);
    }
    for status in statuses {
        match status {
            JobStatus::Queued => bus.emit_job_queued(job.job_id)?,
            JobStatus::Running => bus.emit_job_running(job.job_id)?,
            JobStatus::Succeeded => bus.emit_job_succeeded(job.job_id)?,
            JobStatus::Failed => bus.emit_job_failed(job.job_id)?,
            JobStatus::Cancelled => bus.emit_job_cancelled(job.job_id)?,
        }
    }
    Ok(())
}

#[tauri::command]
pub fn app_get_build_info() -> AppResult<BuildInfo> {
    app::app_get_build_info()
}

#[tauri::command]
pub fn app_get_permissions_status() -> AppResult<PermissionsStatus> {
    app::app_get_permissions_status()
}

#[tauri::command]
pub fn settings_get(state: State<'_, Arc<Backend>>) -> AppResult<AppSettings> {
    app::settings_get(backend(&state))
}

#[tauri::command]
pub fn settings_set(state: State<'_, Arc<Backend>>, req: AppSettings) -> AppResult<AppSettings> {
    app::settings_set(backend(&state), req)
}

#[tauri::command]
pub fn network_allowlist_get(state: State<'_, Arc<Backend>>) -> AppResult<NetworkAllowlist> {
    app::network_allowlist_get(backend(&state))
}

#[tauri::command]
pub fn network_allowlist_set(
    state: State<'_, Arc<Backend>>,
    req: NetworkAllowlistUpdate,
) -> AppResult<NetworkAllowlist> {
    app::network_allowlist_set(backend(&state), req)
}

#[tauri::command]
pub fn session_create(
    state: State<'_, Arc<Backend>>,
    req: SessionCreateRequest,
) -> AppResult<SessionSummary> {
    sessions::session_create(backend(&state), req)
}

#[tauri::command]
pub fn session_list(
    state: State<'_, Arc<Backend>>,
    req: SessionListRequest,
) -> AppResult<Vec<SessionSummary>> {
    sessions::session_list(backend(&state), req)
}

#[tauri::command]
pub fn session_get(
    state: State<'_, Arc<Backend>>,
    req: SessionGetRequest,
) -> AppResult<SessionDetail> {
    sessions::session_get(backend(&state), req)
}

#[tauri::command]
pub fn session_close(
    state: State<'_, Arc<Backend>>,
    req: SessionCloseRequest,
) -> AppResult<SessionSummary> {
    sessions::session_close(backend(&state), req)
}

#[tauri::command]
pub fn timeline_get_keyframes(
    state: State<'_, Arc<Backend>>,
    req: TimelineKeyframesRequest,
) -> AppResult<TimelineKeyframesResponse> {
    timeline::timeline_get_keyframes(backend(&state), req)
}

#[tauri::command]
pub fn timeline_get_events(
    state: State<'_, Arc<Backend>>,
    req: TimelineEventsRequest,
) -> AppResult<TimelineEventsResponse> {
    timeline::timeline_get_events(backend(&state), req)
}

#[tauri::command]
pub fn timeline_get_thumbnail(
    state: State<'_, Arc<Backend>>,
    req: TimelineThumbnailRequest,
) -> AppResult<AssetRef> {
    timeline::timeline_get_thumbnail(backend(&state), req)
}

#[tauri::command]
pub fn capture_get_config() -> AppResult<CaptureConfig> {
    capture::capture_get_config()
}

#[tauri::command]
pub fn capture_set_config(req: CaptureConfig) -> AppResult<CaptureConfig> {
    capture::capture_set_config(req)
}

#[tauri::command]
pub fn capture_start(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: CaptureStartRequest,
) -> AppResult<CaptureStatus> {
    let status = capture::capture_start(backend(&state), req)?;
    if should_emit_capture_status_from_wrapper(backend(&state)) {
        events.emit_capture_status(&status)?;
    }
    Ok(status)
}

#[tauri::command]
pub fn capture_stop(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: CaptureStopRequest,
) -> AppResult<CaptureStatus> {
    let status = capture::capture_stop(backend(&state), req)?;
    if should_emit_capture_status_from_wrapper(backend(&state)) {
        events.emit_capture_status(&status)?;
    }
    Ok(status)
}

#[tauri::command]
pub fn capture_get_status(
    state: State<'_, Arc<Backend>>,
    req: CaptureStatusRequest,
) -> AppResult<CaptureStatus> {
    capture::capture_get_status(backend(&state), req)
}

#[tauri::command]
pub fn ocr_schedule(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: OcrScheduleRequest,
) -> AppResult<JobHandle> {
    let handle = ocr::ocr_schedule(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn ocr_get_status(
    state: State<'_, Arc<Backend>>,
    req: OcrStatusRequest,
) -> AppResult<OcrStatus> {
    ocr::ocr_get_status(backend(&state), req)
}

#[tauri::command]
pub fn ocr_search(
    state: State<'_, Arc<Backend>>,
    req: OcrSearchRequest,
) -> AppResult<OcrSearchResponse> {
    ocr::ocr_search(backend(&state), req)
}

#[tauri::command]
pub fn ocr_get_blocks_for_frame(
    state: State<'_, Arc<Backend>>,
    req: OcrBlocksForFrameRequest,
) -> AppResult<OcrBlocksForFrameResponse> {
    ocr::ocr_get_blocks_for_frame(backend(&state), req)
}

#[tauri::command]
pub fn evidence_for_time_range(
    state: State<'_, Arc<Backend>>,
    req: EvidenceForTimeRangeRequest,
) -> AppResult<EvidenceSet> {
    evidence::evidence_for_time_range(backend(&state), req)
}

#[tauri::command]
pub fn evidence_for_step(
    state: State<'_, Arc<Backend>>,
    req: EvidenceForStepRequest,
) -> AppResult<EvidenceSet> {
    evidence::evidence_for_step(backend(&state), req)
}

#[tauri::command]
pub fn evidence_find_text(
    state: State<'_, Arc<Backend>>,
    req: EvidenceFindTextRequest,
) -> AppResult<EvidenceFindTextResponse> {
    evidence::evidence_find_text(backend(&state), req)
}

#[tauri::command]
pub fn evidence_get_coverage(
    state: State<'_, Arc<Backend>>,
    req: EvidenceCoverageRequest,
) -> AppResult<EvidenceCoverageResponse> {
    evidence::evidence_get_coverage(backend(&state), req)
}

#[tauri::command]
pub fn steps_generate_candidates(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: StepsGenerateCandidatesRequest,
) -> AppResult<JobHandle> {
    let handle = steps::steps_generate_candidates(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn steps_list(
    state: State<'_, Arc<Backend>>,
    req: StepsListRequest,
) -> AppResult<StepsListResponse> {
    steps::steps_list(backend(&state), req)
}

#[tauri::command]
pub fn steps_get(state: State<'_, Arc<Backend>>, req: StepsGetRequest) -> AppResult<StepDetail> {
    steps::steps_get(backend(&state), req)
}

#[tauri::command]
pub fn steps_apply_edit(
    state: State<'_, Arc<Backend>>,
    req: StepsApplyEditRequest,
) -> AppResult<StepsApplyEditResponse> {
    steps::steps_apply_edit(backend(&state), req)
}

#[tauri::command]
pub fn steps_validate(
    state: State<'_, Arc<Backend>>,
    req: StepsValidateRequest,
) -> AppResult<StepsValidateExportResponse> {
    steps::steps_validate(backend(&state), req)
}

#[tauri::command]
pub fn anchors_list_for_step(
    state: State<'_, Arc<Backend>>,
    req: AnchorsListForStepRequest,
) -> AppResult<AnchorsListResponse> {
    anchors::anchors_list_for_step(backend(&state), req)
}

#[tauri::command]
pub fn anchors_reacquire(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: AnchorsReacquireRequest,
) -> AppResult<JobHandle> {
    let handle = anchors::anchors_reacquire(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn anchors_manual_set(
    state: State<'_, Arc<Backend>>,
    req: AnchorsManualSetRequest,
) -> AppResult<AnchorsManualSetResponse> {
    anchors::anchors_manual_set(backend(&state), req)
}

#[tauri::command]
pub fn anchors_debug(
    state: State<'_, Arc<Backend>>,
    req: AnchorsDebugRequest,
) -> AppResult<AnchorsDebugResponse> {
    anchors::anchors_debug(backend(&state), req)
}

#[tauri::command]
pub fn tutorial_generate(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: TutorialGenerateRequest,
) -> AppResult<JobHandle> {
    let handle = slicer::tutorial_generate(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn tutorial_export_pack(
    state: State<'_, Arc<Backend>>,
    req: TutorialExportRequest,
) -> AppResult<ExportResult> {
    slicer::tutorial_export_pack(backend(&state), req)
}

#[tauri::command]
pub fn tutorial_validate_export(
    state: State<'_, Arc<Backend>>,
    req: TutorialValidateExportRequest,
) -> AppResult<TutorialValidateExportResponse> {
    slicer::tutorial_validate_export(backend(&state), req)
}

#[tauri::command]
pub fn explain_this_screen(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: ExplainThisScreenRequest,
) -> AppResult<JobHandle> {
    let handle = slicer::explain_this_screen(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn proof_get_view(
    state: State<'_, Arc<Backend>>,
    req: ProofGetViewRequest,
) -> AppResult<ProofViewResponse> {
    proof::proof_get_view(backend(&state), req)
}

#[tauri::command]
pub fn runbook_create(
    state: State<'_, Arc<Backend>>,
    req: RunbookCreateRequest,
) -> AppResult<RunbookDetail> {
    proof::runbook_create(backend(&state), req)
}

#[tauri::command]
pub fn runbook_update(
    state: State<'_, Arc<Backend>>,
    req: RunbookUpdateRequest,
) -> AppResult<RunbookDetail> {
    proof::runbook_update(backend(&state), req)
}

#[tauri::command]
pub fn runbook_export(
    state: State<'_, Arc<Backend>>,
    req: RunbookExportRequest,
) -> AppResult<ExportResult> {
    proof::runbook_export(backend(&state), req)
}

#[tauri::command]
pub fn proof_export_bundle(
    state: State<'_, Arc<Backend>>,
    req: ProofExportRequest,
) -> AppResult<ExportResult> {
    proof::proof_export_bundle(backend(&state), req)
}

#[tauri::command]
pub fn verifier_list(
    state: State<'_, Arc<Backend>>,
    req: VerifierListRequest,
) -> AppResult<VerifierListResponse> {
    verifiers::verifier_list(backend(&state), req)
}

#[tauri::command]
pub fn verifier_run(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: VerifierRunRequest,
) -> AppResult<JobHandle> {
    let handle = verifiers::verifier_run(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn verifier_get_result(
    state: State<'_, Arc<Backend>>,
    req: VerifierGetResultRequest,
) -> AppResult<VerifierResultDetail> {
    verifiers::verifier_get_result(backend(&state), req)
}

#[tauri::command]
pub fn models_list(
    state: State<'_, Arc<Backend>>,
    req: ModelsListRequest,
) -> AppResult<ModelsListResponse> {
    model_dock::models_list(backend(&state), req)
}

#[tauri::command]
pub fn models_register(
    state: State<'_, Arc<Backend>>,
    req: ModelRegisterRequest,
) -> AppResult<ModelProfile> {
    model_dock::models_register(backend(&state), req)
}

#[tauri::command]
pub fn models_remove(
    state: State<'_, Arc<Backend>>,
    req: ModelsRemoveRequest,
) -> AppResult<ModelsRemoveResponse> {
    model_dock::models_remove(backend(&state), req)
}

#[tauri::command]
pub fn model_roles_get(state: State<'_, Arc<Backend>>) -> AppResult<ModelRoles> {
    model_dock::model_roles_get(backend(&state))
}

#[tauri::command]
pub fn model_roles_set(
    state: State<'_, Arc<Backend>>,
    req: ModelRolesUpdate,
) -> AppResult<ModelRoles> {
    model_dock::model_roles_set(backend(&state), req)
}

#[tauri::command]
pub fn ollama_list(
    state: State<'_, Arc<Backend>>,
    req: OllamaListRequest,
) -> AppResult<OllamaListResponse> {
    model_dock::ollama_list(backend(&state), req)
}

#[tauri::command]
pub fn ollama_pull(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: OllamaPullRequest,
) -> AppResult<JobHandle> {
    let handle = model_dock::ollama_pull(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn ollama_run(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: OllamaRunRequest,
) -> AppResult<JobHandle> {
    let handle = model_dock::ollama_run(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn mlx_run(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: MlxRunRequest,
) -> AppResult<JobHandle> {
    let handle = model_dock::mlx_run(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn bench_run(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: BenchRunRequest,
) -> AppResult<JobHandle> {
    let handle = model_dock::bench_run(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn bench_list(
    state: State<'_, Arc<Backend>>,
    req: BenchListRequest,
) -> AppResult<BenchListResponse> {
    model_dock::bench_list(backend(&state), req)
}

#[tauri::command]
pub fn agent_pipelines_list() -> AppResult<AgentPipelinesListResponse> {
    agent_plant::agent_pipelines_list()
}

#[tauri::command]
pub fn agent_pipeline_run(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: AgentPipelineRunRequest,
) -> AppResult<JobHandle> {
    let handle = agent_plant::agent_pipeline_run(backend(&state), req)?;
    emit_job_lifecycle(events.inner(), backend(&state), &handle)?;
    Ok(handle)
}

#[tauri::command]
pub fn agent_pipeline_report(
    state: State<'_, Arc<Backend>>,
    req: AgentPipelineReportRequest,
) -> AppResult<AgentPipelineReportResponse> {
    agent_plant::agent_pipeline_report(backend(&state), req)
}

#[tauri::command]
pub fn exports_list(
    state: State<'_, Arc<Backend>>,
    req: ExportsListRequest,
) -> AppResult<ExportsListResponse> {
    exports::exports_list(backend(&state), req)
}

#[tauri::command]
pub fn export_verify_bundle(
    state: State<'_, Arc<Backend>>,
    req: ExportVerifyRequest,
) -> AppResult<ExportVerifyResponse> {
    exports::export_verify_bundle(backend(&state), req)
}

#[tauri::command]
pub fn jobs_list(
    state: State<'_, Arc<Backend>>,
    req: JobsListRequest,
) -> AppResult<JobsListResponse> {
    jobs::jobs_list(backend(&state), req)
}

#[tauri::command]
pub fn jobs_get(state: State<'_, Arc<Backend>>, req: JobsGetRequest) -> AppResult<JobDetail> {
    jobs::jobs_get(backend(&state), req)
}

#[tauri::command]
pub fn jobs_cancel(
    events: State<'_, RuntimeEventBus>,
    state: State<'_, Arc<Backend>>,
    req: JobsCancelRequest,
) -> AppResult<JobsCancelResponse> {
    let job_id = req.job_id;
    let res = jobs::jobs_cancel(backend(&state), req)?;
    if res.accepted {
        emit_job_lifecycle(events.inner(), backend(&state), &JobHandle { job_id })?;
    }
    Ok(res)
}

pub fn invoke_handler<R: tauri::Runtime>(
) -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        app_get_build_info,
        app_get_permissions_status,
        settings_get,
        settings_set,
        network_allowlist_get,
        network_allowlist_set,
        session_create,
        session_list,
        session_get,
        session_close,
        timeline_get_keyframes,
        timeline_get_events,
        timeline_get_thumbnail,
        capture_get_config,
        capture_set_config,
        capture_start,
        capture_stop,
        capture_get_status,
        ocr_schedule,
        ocr_get_status,
        ocr_search,
        ocr_get_blocks_for_frame,
        evidence_for_time_range,
        evidence_for_step,
        evidence_find_text,
        evidence_get_coverage,
        steps_generate_candidates,
        steps_list,
        steps_get,
        steps_apply_edit,
        steps_validate,
        anchors_list_for_step,
        anchors_reacquire,
        anchors_manual_set,
        anchors_debug,
        tutorial_generate,
        tutorial_export_pack,
        tutorial_validate_export,
        explain_this_screen,
        proof_get_view,
        runbook_create,
        runbook_update,
        runbook_export,
        proof_export_bundle,
        verifier_list,
        verifier_run,
        verifier_get_result,
        models_list,
        models_register,
        models_remove,
        model_roles_get,
        model_roles_set,
        ollama_list,
        ollama_pull,
        ollama_run,
        mlx_run,
        bench_run,
        bench_list,
        agent_pipelines_list,
        agent_pipeline_run,
        agent_pipeline_report,
        exports_list,
        export_verify_bundle,
        jobs_list,
        jobs_get,
        jobs_cancel
    ]
}
