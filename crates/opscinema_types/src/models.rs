use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub type SessionId = Uuid;
pub type EventId = Uuid;
pub type JobId = Uuid;
pub type StepId = Uuid;
pub type AnchorId = Uuid;
pub type ExportId = Uuid;
pub type RunId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AssetRef {
    pub asset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BuildInfo {
    pub app_name: String,
    pub app_version: String,
    pub commit: String,
    pub built_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PermissionsStatus {
    pub screen_recording: bool,
    pub accessibility: bool,
    pub full_disk_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AppSettings {
    pub offline_mode: bool,
    pub allow_input_capture: bool,
    pub allow_window_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct NetworkAllowlist {
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct NetworkAllowlistUpdate {
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub head_seq: i64,
    pub head_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SessionDetail {
    pub summary: SessionSummary,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TimelineEvent {
    pub seq: i64,
    pub event_id: EventId,
    pub event_type: String,
    pub frame_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TimelineKeyframe {
    pub frame_ms: i64,
    pub frame_event_id: EventId,
    pub asset: AssetRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CaptureConfig {
    pub keyframe_interval_ms: u32,
    pub include_input: bool,
    pub include_window_meta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CaptureState {
    Idle,
    Capturing,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CaptureStatus {
    pub state: CaptureState,
    pub session_id: Option<SessionId>,
    pub started_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobHandle {
    pub job_id: JobId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobCounters {
    pub done: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobProgress {
    pub stage: String,
    pub pct: u8,
    pub counters: JobCounters,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct JobDetail {
    pub job_id: JobId,
    pub job_type: String,
    pub session_id: Option<SessionId>,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub progress: Option<JobProgress>,
    pub error: Option<crate::AppError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct JobsListResponse {
    pub jobs: Vec<JobDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobsCancelResponse {
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrBlock {
    pub ocr_block_id: String,
    pub bbox_norm: BBoxNorm,
    pub text: String,
    pub confidence: u8,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrStatus {
    pub queued_frames: u32,
    pub indexed_frames: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrSearchHit {
    pub frame_ms: i64,
    pub block_id: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrSearchResponse {
    pub hits: Vec<OcrSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrBlocksForFrameResponse {
    pub blocks: Vec<OcrBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceSet {
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceItem {
    pub evidence_id: Uuid,
    pub kind: String,
    pub source_id: String,
    pub locators: Vec<EvidenceLocator>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLocatorType {
    Timeline,
    FrameBbox,
    OcrBbox,
    AnchorBbox,
    VerifierLog,
    FilePath,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TextOffset {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BBoxNorm {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceLocator {
    pub locator_type: EvidenceLocatorType,
    pub asset_id: Option<String>,
    pub frame_ms: Option<i64>,
    pub bbox_norm: Option<BBoxNorm>,
    pub text_offset: Option<TextOffset>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceFindTextResponse {
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EvidenceCoverageResponse {
    pub missing_step_ids: Vec<StepId>,
    pub missing_generated_block_ids: Vec<String>,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextBlockProvenance {
    Human,
    Generated,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TextBlock {
    pub block_id: String,
    pub text: String,
    pub provenance: TextBlockProvenance,
    pub evidence_refs: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StructuredText {
    pub blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Step {
    pub step_id: StepId,
    pub order_index: u32,
    pub title: String,
    pub body: StructuredText,
    pub risk_tags: Vec<String>,
    pub branch_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsListResponse {
    pub steps: Vec<Step>,
    pub head_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepDetail {
    pub step: Step,
    pub anchors: Vec<AnchorCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepEditOp {
    InsertAfter {
        after_step_id: StepId,
        step: Step,
    },
    UpdateTitle {
        step_id: StepId,
        title: String,
    },
    ReplaceBody {
        step_id: StepId,
        body: StructuredText,
    },
    Delete {
        step_id: StepId,
    },
    Reorder {
        step_id: StepId,
        new_index: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsApplyEditResponse {
    pub head_seq: i64,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepsValidateExportResponse {
    pub schema_valid: bool,
    pub evidence_valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnchorKind {
    UiTarget,
    OcrPhrase,
    VisionAnchor,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorCandidate {
    pub anchor_id: AnchorId,
    pub step_id: StepId,
    pub kind: AnchorKind,
    pub target_signature: String,
    pub confidence: u8,
    pub locators: Vec<EvidenceLocator>,
    pub degraded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorsListResponse {
    pub anchors: Vec<AnchorCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorsManualSetResponse {
    pub anchor: AnchorCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AnchorsDebugResponse {
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportResult {
    pub export_id: ExportId,
    pub output_path: String,
    pub bundle_hash: String,
    pub warnings: Vec<ExportWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TutorialValidateExportResponse {
    pub allowed: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProofViewResponse {
    pub steps: Vec<Step>,
    pub evidence: EvidenceSet,
    pub warnings: Vec<ExportWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RunbookDetail {
    pub runbook_id: Uuid,
    pub title: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierSpec {
    pub verifier_id: String,
    pub kind: String,
    pub timeout_secs: u32,
    pub command_allowlist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierListResponse {
    pub verifiers: Vec<VerifierSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierResultDetail {
    pub run_id: RunId,
    pub verifier_id: String,
    pub status: String,
    pub result_asset: AssetRef,
    pub logs_asset: Option<AssetRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelProfile {
    pub model_id: String,
    pub provider: String,
    pub label: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelsListResponse {
    pub models: Vec<ModelProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelsRemoveResponse {
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelRoles {
    pub tutorial_generation: Option<String>,
    pub screen_explainer: Option<String>,
    pub anchor_grounding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelRolesUpdate {
    pub tutorial_generation: Option<String>,
    pub screen_explainer: Option<String>,
    pub anchor_grounding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OllamaListResponse {
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchRecord {
    pub bench_id: Uuid,
    pub model_id: String,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchListResponse {
    pub benches: Vec<BenchRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AgentPipelinesListResponse {
    pub pipelines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AgentPipelineReportResponse {
    pub run_id: Uuid,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportsListResponse {
    pub exports: Vec<ExportResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportVerifyResponse {
    pub valid: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TimelineKeyframesResponse {
    pub keyframes: Vec<TimelineKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TimelineEventsResponse {
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EventStreamEnvelope<T> {
    pub stream_seq: u64,
    pub sent_at: DateTime<Utc>,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobProgressEvent {
    pub job_id: JobId,
    pub stage: String,
    pub pct: u8,
    pub counters: JobCounters,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct JobStatusEvent {
    pub job_id: JobId,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CaptureStatusEvent {
    pub state: CaptureState,
    pub session_id: Option<SessionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StepModel {
    pub schema_version: u32,
    pub steps: Vec<Step>,
}
