# Contracts: IPC, Errors, Jobs, UI State Approach (Phase 0 Lock)

## 1) Error Envelope (Global)
Every IPC call returns `Result<T, AppError>`.

`AppError` fields (stable):
- `code: AppErrorCode` (enum -> string)
- `message: string` (user-safe)
- `details?: string` (developer diagnostics; local logs)
- `recoverable: bool`
- `action_hint?: string` (UI navigation hint)

`AppErrorCode` (stable set):
- `PERMISSION_DENIED`
- `VALIDATION_FAILED`
- `NOT_FOUND`
- `CONFLICT`
- `POLICY_BLOCKED`
- `NETWORK_BLOCKED`
- `EXPORT_GATE_FAILED`
- `PROVIDER_SCHEMA_INVALID`
- `IO`
- `DB`
- `JOB_CANCELLED`
- `UNSUPPORTED`
- `INTERNAL`

Mapping rules:
- Missing permissions -> `PERMISSION_DENIED` + action_hint
- Export blocked by evidence/anchors -> `EXPORT_GATE_FAILED`
- Network attempt not allowlisted -> `NETWORK_BLOCKED`

## 2) Job Model (Global)
All expensive tasks are jobs:
- OCR
- Step candidate generation
- Anchor reacquire / vision grounding
- Exports
- Verifier runs
- Benchmarks
- Agent pipeline runs

Job fields:
- `job_id`
- `job_type`
- `session_id?`
- `created_at`, `started_at?`, `ended_at?`
- `status: QUEUED | RUNNING | SUCCEEDED | FAILED | CANCELLED`
- `progress` (typed per job_type)
- `error?: AppError` (for FAILED)

Cancellation:
- `jobs_cancel(job_id)` is idempotent.
- job runners must check cancellation between units of work.
- partial results must be persisted safely; job must never leave dangling references.

Progress events:
- emitted as typed IPC events `job_progress` and `job_status`.
- progress payload includes:
  - `job_id`
  - `stage` (stable string)
  - `pct` (0..100)
  - `counters` (e.g., frames_done/frames_total)

## 3) Typed IPC Lock (Phase 0)
### Rules
- IPC command list is locked in Phase 0.
- Requests/responses are defined in Rust and codegen’d to TS.
- UI must not use untyped `invoke()` directly; only generated client.

### Events
- `job_progress`
- `job_status`
- `capture_status`

All events include:
- `stream_seq` (monotonic per stream within process)
- `sent_at` (UTC ISO8601)

## 4) IPC Command List (Locked)
This list is the Phase 0 lock. Do not add commands later without major IPC contract bump.

**App + Settings**
- `app_get_build_info() -> BuildInfo`
- `app_get_permissions_status() -> PermissionsStatus`
- `settings_get() -> AppSettings`
- `settings_set(AppSettings) -> AppSettings`
- `network_allowlist_get() -> NetworkAllowlist`
- `network_allowlist_set(NetworkAllowlistUpdate) -> NetworkAllowlist`

**Sessions**
- `session_create(SessionCreateRequest) -> SessionSummary`
- `session_list(SessionListRequest) -> Vec<SessionSummary>`
- `session_get(SessionGetRequest) -> SessionDetail`
- `session_close(SessionCloseRequest) -> SessionSummary`

**Timeline**
- `timeline_get_keyframes(TimelineKeyframesRequest) -> TimelineKeyframesResponse`
- `timeline_get_events(TimelineEventsRequest) -> TimelineEventsResponse`
- `timeline_get_thumbnail(TimelineThumbnailRequest) -> AssetRef`

**Capture**
- `capture_get_config() -> CaptureConfig`
- `capture_set_config(CaptureConfig) -> CaptureConfig`
- `capture_start(CaptureStartRequest) -> CaptureStatus`
- `capture_stop(CaptureStopRequest) -> CaptureStatus`
- `capture_get_status(CaptureStatusRequest) -> CaptureStatus`

**OCR**
- `ocr_schedule(OcrScheduleRequest) -> JobHandle`
- `ocr_get_status(OcrStatusRequest) -> OcrStatus`
- `ocr_search(OcrSearchRequest) -> OcrSearchResponse`
- `ocr_get_blocks_for_frame(OcrBlocksForFrameRequest) -> OcrBlocksForFrameResponse`

**Evidence**
- `evidence_for_time_range(EvidenceForTimeRangeRequest) -> EvidenceSet`
- `evidence_for_step(EvidenceForStepRequest) -> EvidenceSet`
- `evidence_find_text(EvidenceFindTextRequest) -> EvidenceFindTextResponse`
- `evidence_get_coverage(EvidenceCoverageRequest) -> EvidenceCoverageResponse`

**Steps**
- `steps_generate_candidates(StepsGenerateCandidatesRequest) -> JobHandle`
- `steps_list(StepsListRequest) -> StepsListResponse`
- `steps_get(StepsGetRequest) -> StepDetail`
- `steps_apply_edit(StepsApplyEditRequest) -> StepsApplyEditResponse`
- `steps_validate(StepsValidateRequest) -> StepsValidateExportResponse`

**Anchors**
- `anchors_list_for_step(AnchorsListForStepRequest) -> AnchorsListResponse`
- `anchors_reacquire(AnchorsReacquireRequest) -> JobHandle`
- `anchors_manual_set(AnchorsManualSetRequest) -> AnchorsManualSetResponse`
- `anchors_debug(AnchorsDebugRequest) -> AnchorsDebugResponse`

**Slicer Studio**
- `tutorial_generate(TutorialGenerateRequest) -> JobHandle`
- `tutorial_export_pack(TutorialExportRequest) -> ExportResult`
- `tutorial_validate_export(TutorialValidateExportRequest) -> TutorialValidateExportResponse`
- `explain_this_screen(ExplainThisScreenRequest) -> JobHandle`

**Proof / Runbooks / Verifiers**
- `proof_get_view(ProofGetViewRequest) -> ProofViewResponse`
- `runbook_create(RunbookCreateRequest) -> RunbookDetail`
- `runbook_update(RunbookUpdateRequest) -> RunbookDetail`
- `runbook_export(RunbookExportRequest) -> ExportResult`
- `proof_export_bundle(ProofExportRequest) -> ExportResult`
- `verifier_list(VerifierListRequest) -> VerifierListResponse`
- `verifier_run(VerifierRunRequest) -> JobHandle`
- `verifier_get_result(VerifierGetResultRequest) -> VerifierResultDetail`

**Model Dock**
- `models_list(ModelsListRequest) -> ModelsListResponse`
- `models_register(ModelRegisterRequest) -> ModelProfile`
- `models_remove(ModelsRemoveRequest) -> ModelsRemoveResponse`
- `model_roles_get() -> ModelRoles`
- `model_roles_set(ModelRolesUpdate) -> ModelRoles`
- `ollama_list(OllamaListRequest) -> OllamaListResponse`
- `ollama_pull(OllamaPullRequest) -> JobHandle`
- `ollama_run(OllamaRunRequest) -> JobHandle`
- `mlx_run(MlxRunRequest) -> JobHandle`
- `bench_run(BenchRunRequest) -> JobHandle`
- `bench_list(BenchListRequest) -> BenchListResponse`

**Agent Plant (Internal)**
- `agent_pipelines_list() -> AgentPipelinesListResponse`
- `agent_pipeline_run(AgentPipelineRunRequest) -> JobHandle`
- `agent_pipeline_report(AgentPipelineReportRequest) -> AgentPipelineReportResponse`

**Exports**
- `exports_list(ExportsListRequest) -> ExportsListResponse`
- `export_verify_bundle(ExportVerifyRequest) -> ExportVerifyResponse`

**Jobs**
- `jobs_list(JobsListRequest) -> JobsListResponse`
- `jobs_get(JobsGetRequest) -> JobDetail`
- `jobs_cancel(JobsCancelRequest) -> JobsCancelResponse`

## 5) UI State Management Approach (Explicit)
UI is a pure client:
- It fetches authoritative state via IPC.
- It may cache responses in-memory (per view) and invalidate on:
  - job status events
  - session head seq changes
- Optimistic edits:
  - include `base_seq` (session head seq) in edit requests
  - handle `CONFLICT` by refetching steps and reapplying (UI-level)

No UI persistence layer is permitted.

## 6) Contract Tests (Non-Optional)
- Rust serialization round-trip tests for all IPC request/response types.
- TS compile step that fails if generated bindings contain `any`.
- Runtime smoke:
  - `app_get_build_info`
  - `session_create` -> `session_get`
  - error envelope: intentionally trigger `PERMISSION_DENIED` path in a harness and validate fields.
