use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::models::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SessionCreateRequest {
    pub label: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SessionListRequest {
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SessionGetRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SessionCloseRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TimelineKeyframesRequest {
    pub session_id: SessionId,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TimelineEventsRequest {
    pub session_id: SessionId,
    pub after_seq: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TimelineThumbnailRequest {
    pub session_id: SessionId,
    pub frame_event_id: EventId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CaptureStartRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CaptureStopRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CaptureStatusRequest {
    pub session_id: Option<SessionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrScheduleRequest {
    pub session_id: SessionId,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrStatusRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrSearchRequest {
    pub session_id: SessionId,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrBlocksForFrameRequest {
    pub session_id: SessionId,
    pub frame_event_id: EventId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceForTimeRangeRequest {
    pub session_id: SessionId,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceForStepRequest {
    pub session_id: SessionId,
    pub step_id: StepId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceFindTextRequest {
    pub session_id: SessionId,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceCoverageRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsGenerateCandidatesRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsListRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsGetRequest {
    pub session_id: SessionId,
    pub step_id: StepId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsApplyEditRequest {
    pub session_id: SessionId,
    pub base_seq: i64,
    pub op: StepEditOp,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsValidateRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorsListForStepRequest {
    pub session_id: SessionId,
    pub step_id: StepId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorsReacquireRequest {
    pub session_id: SessionId,
    pub step_id: StepId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorsManualSetRequest {
    pub session_id: SessionId,
    pub anchor_id: AnchorId,
    pub locators: Vec<EvidenceLocator>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorsDebugRequest {
    pub session_id: SessionId,
    pub step_id: StepId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TutorialGenerateRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TutorialExportRequest {
    pub session_id: SessionId,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TutorialValidateExportRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExplainThisScreenRequest {
    pub session_id: SessionId,
    pub frame_event_id: EventId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProofGetViewRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RunbookCreateRequest {
    pub session_id: SessionId,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RunbookUpdateRequest {
    pub runbook_id: Uuid,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RunbookExportRequest {
    pub runbook_id: Uuid,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProofExportRequest {
    pub session_id: SessionId,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierListRequest {
    pub include_disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierRunRequest {
    pub session_id: SessionId,
    pub verifier_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierGetResultRequest {
    pub run_id: RunId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelsListRequest {
    pub include_unhealthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelRegisterRequest {
    pub provider: String,
    pub label: String,
    pub model_name: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelsRemoveRequest {
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OllamaListRequest {
    pub host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OllamaPullRequest {
    pub host: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OllamaRunRequest {
    pub model_id: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct MlxRunRequest {
    pub model_id: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchRunRequest {
    pub model_id: String,
    pub benchmark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchListRequest {
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AgentPipelineRunRequest {
    pub session_id: SessionId,
    pub pipeline_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AgentPipelineReportRequest {
    pub run_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportsListRequest {
    pub session_id: Option<SessionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportVerifyRequest {
    pub bundle_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobsListRequest {
    pub session_id: Option<SessionId>,
    pub status: Option<JobStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobsGetRequest {
    pub job_id: JobId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobsCancelRequest {
    pub job_id: JobId,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum IpcCommand {
    AppGetBuildInfo,
    AppGetPermissionsStatus,
    SettingsGet,
    SettingsSet,
    NetworkAllowlistGet,
    NetworkAllowlistSet,
    SessionCreate,
    SessionList,
    SessionGet,
    SessionClose,
    TimelineGetKeyframes,
    TimelineGetEvents,
    TimelineGetThumbnail,
    CaptureGetConfig,
    CaptureSetConfig,
    CaptureStart,
    CaptureStop,
    CaptureGetStatus,
    OcrSchedule,
    OcrGetStatus,
    OcrSearch,
    OcrGetBlocksForFrame,
    EvidenceForTimeRange,
    EvidenceForStep,
    EvidenceFindText,
    EvidenceGetCoverage,
    StepsGenerateCandidates,
    StepsList,
    StepsGet,
    StepsApplyEdit,
    StepsValidate,
    AnchorsListForStep,
    AnchorsReacquire,
    AnchorsManualSet,
    AnchorsDebug,
    TutorialGenerate,
    TutorialExportPack,
    TutorialValidateExport,
    ExplainThisScreen,
    ProofGetView,
    RunbookCreate,
    RunbookUpdate,
    RunbookExport,
    ProofExportBundle,
    VerifierList,
    VerifierRun,
    VerifierGetResult,
    ModelsList,
    ModelsRegister,
    ModelsRemove,
    ModelRolesGet,
    ModelRolesSet,
    OllamaList,
    OllamaPull,
    OllamaRun,
    MlxRun,
    BenchRun,
    BenchList,
    AgentPipelinesList,
    AgentPipelineRun,
    AgentPipelineReport,
    ExportsList,
    ExportVerifyBundle,
    JobsList,
    JobsGet,
    JobsCancel,
}

impl IpcCommand {
    pub const LOCKED_COMMANDS: &'static [IpcCommand] = &[
        IpcCommand::AppGetBuildInfo,
        IpcCommand::AppGetPermissionsStatus,
        IpcCommand::SettingsGet,
        IpcCommand::SettingsSet,
        IpcCommand::NetworkAllowlistGet,
        IpcCommand::NetworkAllowlistSet,
        IpcCommand::SessionCreate,
        IpcCommand::SessionList,
        IpcCommand::SessionGet,
        IpcCommand::SessionClose,
        IpcCommand::TimelineGetKeyframes,
        IpcCommand::TimelineGetEvents,
        IpcCommand::TimelineGetThumbnail,
        IpcCommand::CaptureGetConfig,
        IpcCommand::CaptureSetConfig,
        IpcCommand::CaptureStart,
        IpcCommand::CaptureStop,
        IpcCommand::CaptureGetStatus,
        IpcCommand::OcrSchedule,
        IpcCommand::OcrGetStatus,
        IpcCommand::OcrSearch,
        IpcCommand::OcrGetBlocksForFrame,
        IpcCommand::EvidenceForTimeRange,
        IpcCommand::EvidenceForStep,
        IpcCommand::EvidenceFindText,
        IpcCommand::EvidenceGetCoverage,
        IpcCommand::StepsGenerateCandidates,
        IpcCommand::StepsList,
        IpcCommand::StepsGet,
        IpcCommand::StepsApplyEdit,
        IpcCommand::StepsValidate,
        IpcCommand::AnchorsListForStep,
        IpcCommand::AnchorsReacquire,
        IpcCommand::AnchorsManualSet,
        IpcCommand::AnchorsDebug,
        IpcCommand::TutorialGenerate,
        IpcCommand::TutorialExportPack,
        IpcCommand::TutorialValidateExport,
        IpcCommand::ExplainThisScreen,
        IpcCommand::ProofGetView,
        IpcCommand::RunbookCreate,
        IpcCommand::RunbookUpdate,
        IpcCommand::RunbookExport,
        IpcCommand::ProofExportBundle,
        IpcCommand::VerifierList,
        IpcCommand::VerifierRun,
        IpcCommand::VerifierGetResult,
        IpcCommand::ModelsList,
        IpcCommand::ModelsRegister,
        IpcCommand::ModelsRemove,
        IpcCommand::ModelRolesGet,
        IpcCommand::ModelRolesSet,
        IpcCommand::OllamaList,
        IpcCommand::OllamaPull,
        IpcCommand::OllamaRun,
        IpcCommand::MlxRun,
        IpcCommand::BenchRun,
        IpcCommand::BenchList,
        IpcCommand::AgentPipelinesList,
        IpcCommand::AgentPipelineRun,
        IpcCommand::AgentPipelineReport,
        IpcCommand::ExportsList,
        IpcCommand::ExportVerifyBundle,
        IpcCommand::JobsList,
        IpcCommand::JobsGet,
        IpcCommand::JobsCancel,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            IpcCommand::AppGetBuildInfo => "app_get_build_info",
            IpcCommand::AppGetPermissionsStatus => "app_get_permissions_status",
            IpcCommand::SettingsGet => "settings_get",
            IpcCommand::SettingsSet => "settings_set",
            IpcCommand::NetworkAllowlistGet => "network_allowlist_get",
            IpcCommand::NetworkAllowlistSet => "network_allowlist_set",
            IpcCommand::SessionCreate => "session_create",
            IpcCommand::SessionList => "session_list",
            IpcCommand::SessionGet => "session_get",
            IpcCommand::SessionClose => "session_close",
            IpcCommand::TimelineGetKeyframes => "timeline_get_keyframes",
            IpcCommand::TimelineGetEvents => "timeline_get_events",
            IpcCommand::TimelineGetThumbnail => "timeline_get_thumbnail",
            IpcCommand::CaptureGetConfig => "capture_get_config",
            IpcCommand::CaptureSetConfig => "capture_set_config",
            IpcCommand::CaptureStart => "capture_start",
            IpcCommand::CaptureStop => "capture_stop",
            IpcCommand::CaptureGetStatus => "capture_get_status",
            IpcCommand::OcrSchedule => "ocr_schedule",
            IpcCommand::OcrGetStatus => "ocr_get_status",
            IpcCommand::OcrSearch => "ocr_search",
            IpcCommand::OcrGetBlocksForFrame => "ocr_get_blocks_for_frame",
            IpcCommand::EvidenceForTimeRange => "evidence_for_time_range",
            IpcCommand::EvidenceForStep => "evidence_for_step",
            IpcCommand::EvidenceFindText => "evidence_find_text",
            IpcCommand::EvidenceGetCoverage => "evidence_get_coverage",
            IpcCommand::StepsGenerateCandidates => "steps_generate_candidates",
            IpcCommand::StepsList => "steps_list",
            IpcCommand::StepsGet => "steps_get",
            IpcCommand::StepsApplyEdit => "steps_apply_edit",
            IpcCommand::StepsValidate => "steps_validate",
            IpcCommand::AnchorsListForStep => "anchors_list_for_step",
            IpcCommand::AnchorsReacquire => "anchors_reacquire",
            IpcCommand::AnchorsManualSet => "anchors_manual_set",
            IpcCommand::AnchorsDebug => "anchors_debug",
            IpcCommand::TutorialGenerate => "tutorial_generate",
            IpcCommand::TutorialExportPack => "tutorial_export_pack",
            IpcCommand::TutorialValidateExport => "tutorial_validate_export",
            IpcCommand::ExplainThisScreen => "explain_this_screen",
            IpcCommand::ProofGetView => "proof_get_view",
            IpcCommand::RunbookCreate => "runbook_create",
            IpcCommand::RunbookUpdate => "runbook_update",
            IpcCommand::RunbookExport => "runbook_export",
            IpcCommand::ProofExportBundle => "proof_export_bundle",
            IpcCommand::VerifierList => "verifier_list",
            IpcCommand::VerifierRun => "verifier_run",
            IpcCommand::VerifierGetResult => "verifier_get_result",
            IpcCommand::ModelsList => "models_list",
            IpcCommand::ModelsRegister => "models_register",
            IpcCommand::ModelsRemove => "models_remove",
            IpcCommand::ModelRolesGet => "model_roles_get",
            IpcCommand::ModelRolesSet => "model_roles_set",
            IpcCommand::OllamaList => "ollama_list",
            IpcCommand::OllamaPull => "ollama_pull",
            IpcCommand::OllamaRun => "ollama_run",
            IpcCommand::MlxRun => "mlx_run",
            IpcCommand::BenchRun => "bench_run",
            IpcCommand::BenchList => "bench_list",
            IpcCommand::AgentPipelinesList => "agent_pipelines_list",
            IpcCommand::AgentPipelineRun => "agent_pipeline_run",
            IpcCommand::AgentPipelineReport => "agent_pipeline_report",
            IpcCommand::ExportsList => "exports_list",
            IpcCommand::ExportVerifyBundle => "export_verify_bundle",
            IpcCommand::JobsList => "jobs_list",
            IpcCommand::JobsGet => "jobs_get",
            IpcCommand::JobsCancel => "jobs_cancel",
        }
    }
}
