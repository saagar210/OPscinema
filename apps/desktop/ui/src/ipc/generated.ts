// GENERATED FILE - DO NOT EDIT

export type JsonObject = { [key: string]: JsonValue };
export type JsonValue = string | number | boolean | null | JsonObject | JsonValue[];

export type AppResult<T> = { ok: true; value: T } | { ok: false; error: AppError };
export type AppErrorCode =
  | 'PERMISSION_DENIED'
  | 'VALIDATION_FAILED'
  | 'NOT_FOUND'
  | 'CONFLICT'
  | 'POLICY_BLOCKED'
  | 'NETWORK_BLOCKED'
  | 'EXPORT_GATE_FAILED'
  | 'PROVIDER_SCHEMA_INVALID'
  | 'IO'
  | 'DB'
  | 'JOB_CANCELLED'
  | 'UNSUPPORTED'
  | 'INTERNAL';
export interface AppError { code: AppErrorCode; message: string; details?: string; recoverable: boolean; action_hint?: string; }

export type IpcCommand =
  'app_get_build_info' |
  'app_get_permissions_status' |
  'settings_get' |
  'settings_set' |
  'network_allowlist_get' |
  'network_allowlist_set' |
  'session_create' |
  'session_list' |
  'session_get' |
  'session_close' |
  'timeline_get_keyframes' |
  'timeline_get_events' |
  'timeline_get_thumbnail' |
  'capture_get_config' |
  'capture_set_config' |
  'capture_start' |
  'capture_stop' |
  'capture_get_status' |
  'ocr_schedule' |
  'ocr_get_status' |
  'ocr_search' |
  'ocr_get_blocks_for_frame' |
  'evidence_for_time_range' |
  'evidence_for_step' |
  'evidence_find_text' |
  'evidence_get_coverage' |
  'steps_generate_candidates' |
  'steps_list' |
  'steps_get' |
  'steps_apply_edit' |
  'steps_validate' |
  'anchors_list_for_step' |
  'anchors_reacquire' |
  'anchors_manual_set' |
  'anchors_debug' |
  'tutorial_generate' |
  'tutorial_export_pack' |
  'tutorial_validate_export' |
  'explain_this_screen' |
  'proof_get_view' |
  'runbook_create' |
  'runbook_update' |
  'runbook_export' |
  'proof_export_bundle' |
  'verifier_list' |
  'verifier_run' |
  'verifier_get_result' |
  'models_list' |
  'models_register' |
  'models_remove' |
  'model_roles_get' |
  'model_roles_set' |
  'ollama_list' |
  'ollama_pull' |
  'ollama_run' |
  'mlx_run' |
  'bench_run' |
  'bench_list' |
  'agent_pipelines_list' |
  'agent_pipeline_run' |
  'agent_pipeline_report' |
  'exports_list' |
  'export_verify_bundle' |
  'jobs_list' |
  'jobs_get' |
  'jobs_cancel';

export interface IpcClient {
  invoke<TReq, TRes>(command: IpcCommand, payload: TReq): Promise<AppResult<TRes>>;
}

export interface IpcRequestMap {
  'app_get_build_info': Record<string, never>;
  'app_get_permissions_status': Record<string, never>;
  'settings_get': Record<string, never>;
  'settings_set': { offline_mode: boolean; allow_input_capture: boolean; allow_window_metadata: boolean };
  'network_allowlist_get': Record<string, never>;
  'network_allowlist_set': { entries: string[] };
  'session_create': { label: string; metadata: Record<string, string> };
  'session_list': { limit?: number };
  'session_get': { session_id: string };
  'session_close': { session_id: string };
  'timeline_get_keyframes': { session_id: string; start_ms: number; end_ms: number };
  'timeline_get_events': { session_id: string; after_seq?: number; limit?: number };
  'timeline_get_thumbnail': { session_id: string; frame_event_id: string };
  'capture_get_config': Record<string, never>;
  'capture_set_config': { keyframe_interval_ms: number; include_input: boolean; include_window_meta: boolean };
  'capture_start': { session_id: string };
  'capture_stop': { session_id: string };
  'capture_get_status': { session_id?: string };
  'ocr_schedule': { session_id: string; start_ms?: number; end_ms?: number };
  'ocr_get_status': { session_id: string };
  'ocr_search': { session_id: string; query: string };
  'ocr_get_blocks_for_frame': { session_id: string; frame_event_id: string };
  'evidence_for_time_range': { session_id: string; start_ms: number; end_ms: number };
  'evidence_for_step': { session_id: string; step_id: string };
  'evidence_find_text': { session_id: string; query: string };
  'evidence_get_coverage': { session_id: string };
  'steps_generate_candidates': { session_id: string };
  'steps_list': { session_id: string };
  'steps_get': { session_id: string; step_id: string };
  'steps_apply_edit': { session_id: string; base_seq: number; op: JsonObject };
  'steps_validate': { session_id: string };
  'anchors_list_for_step': { session_id: string; step_id: string };
  'anchors_reacquire': { session_id: string; step_id: string };
  'anchors_manual_set': { session_id: string; anchor_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }>; note?: string };
  'anchors_debug': { session_id: string; step_id: string };
  'tutorial_generate': { session_id: string };
  'tutorial_export_pack': { session_id: string; output_dir: string };
  'tutorial_validate_export': { session_id: string };
  'explain_this_screen': { session_id: string; frame_event_id: string };
  'proof_get_view': { session_id: string };
  'runbook_create': { session_id: string; title: string };
  'runbook_update': { runbook_id: string; title?: string };
  'runbook_export': { runbook_id?: string; session_id?: string; output_dir: string };
  'proof_export_bundle': { runbook_id?: string; session_id?: string; output_dir: string };
  'verifier_list': { include_disabled: boolean };
  'verifier_run': { session_id: string; verifier_id: string };
  'verifier_get_result': { run_id: string };
  'models_list': { include_unhealthy: boolean };
  'models_register': { provider: string; label: string; model_name: string; digest: string };
  'models_remove': { model_id: string };
  'model_roles_get': Record<string, never>;
  'model_roles_set': { tutorial_generation?: string; screen_explainer?: string; anchor_grounding?: string };
  'ollama_list': { host?: string };
  'ollama_pull': { host?: string; model: string };
  'ollama_run': { model_id: string; prompt: string };
  'mlx_run': { model_id: string; prompt: string };
  'bench_run': { model_id: string; benchmark: string };
  'bench_list': { limit?: number };
  'agent_pipelines_list': Record<string, never>;
  'agent_pipeline_run': { session_id: string; pipeline_id: string };
  'agent_pipeline_report': { run_id: string };
  'exports_list': { session_id?: string };
  'export_verify_bundle': { bundle_path: string };
  'jobs_list': { session_id?: string; status?: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED' };
  'jobs_get': { job_id: string };
  'jobs_cancel': { job_id: string };
}

export interface IpcResponseMap {
  'app_get_build_info': { app_name: string; app_version: string; commit: string; built_at: string };
  'app_get_permissions_status': { screen_recording: boolean; accessibility: boolean; full_disk_access: boolean };
  'settings_get': { offline_mode: boolean; allow_input_capture: boolean; allow_window_metadata: boolean };
  'settings_set': { offline_mode: boolean; allow_input_capture: boolean; allow_window_metadata: boolean };
  'network_allowlist_get': { entries: string[] };
  'network_allowlist_set': { entries: string[] };
  'session_create': { session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string };
  'session_list': Array<{ session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string }>;
  'session_get': { summary: { session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string }; metadata: Record<string, string> };
  'session_close': { session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string };
  'timeline_get_keyframes': { keyframes: Array<{ frame_event_id: string; frame_ms: number; asset: { asset_id: string } }> };
  'timeline_get_events': { events: Array<{ seq: number; event_id: string; event_type: string; frame_ms?: number }> };
  'timeline_get_thumbnail': { asset_id: string };
  'capture_get_config': { keyframe_interval_ms: number; include_input: boolean; include_window_meta: boolean };
  'capture_set_config': { keyframe_interval_ms: number; include_input: boolean; include_window_meta: boolean };
  'capture_start': { state: 'IDLE' | 'CAPTURING' | 'STOPPED'; session_id?: string; started_at?: string };
  'capture_stop': { state: 'IDLE' | 'CAPTURING' | 'STOPPED'; session_id?: string; started_at?: string };
  'capture_get_status': { state: 'IDLE' | 'CAPTURING' | 'STOPPED'; session_id?: string; started_at?: string };
  'ocr_schedule': { job_id: string };
  'ocr_get_status': { queued_frames: number; indexed_frames: number };
  'ocr_search': { hits: Array<{ frame_ms: number; block_id: string; snippet: string }> };
  'ocr_get_blocks_for_frame': { blocks: Array<{ ocr_block_id: string; bbox_norm: { x: number; y: number; w: number; h: number }; text: string; confidence: number; language?: string }> };
  'evidence_for_time_range': { evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> };
  'evidence_for_step': { evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> };
  'evidence_find_text': { evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> };
  'evidence_get_coverage': { missing_step_ids: string[]; missing_generated_block_ids: string[]; pass: boolean };
  'steps_generate_candidates': { job_id: string };
  'steps_list': { steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }>; head_seq: number };
  'steps_get': { step: { step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }; anchors: Array<{ anchor_id: string; step_id: string; kind: string; target_signature: string; confidence: number; degraded: boolean; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> };
  'steps_apply_edit': { head_seq: number; applied: boolean };
  'steps_validate': { schema_valid: boolean; evidence_valid: boolean; errors: string[] };
  'anchors_list_for_step': { anchors: Array<{ anchor_id: string; step_id: string; kind: string; target_signature: string; confidence: number; degraded: boolean; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> };
  'anchors_reacquire': { job_id: string };
  'anchors_manual_set': { anchor: { anchor_id: string; step_id: string; kind: string; target_signature: string; confidence: number; degraded: boolean; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> } };
  'anchors_debug': { checks: string[] };
  'tutorial_generate': { job_id: string };
  'tutorial_export_pack': { export_id: string; output_path: string; bundle_hash: string; warnings: Array<{ code: string; message: string }> };
  'tutorial_validate_export': { allowed: boolean; reasons: string[] };
  'explain_this_screen': { job_id: string };
  'proof_get_view': { steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }>; evidence: { evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> }; warnings: Array<{ code: string; message: string }> };
  'runbook_create': { runbook_id: string; title: string; steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }> };
  'runbook_update': { runbook_id: string; title: string; steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }> };
  'runbook_export': { export_id: string; output_path: string; bundle_hash: string; warnings: Array<{ code: string; message: string }> };
  'proof_export_bundle': { export_id: string; output_path: string; bundle_hash: string; warnings: Array<{ code: string; message: string }> };
  'verifier_list': { verifiers: Array<{ verifier_id: string; kind: string; timeout_secs: number; command_allowlist: string[] }> };
  'verifier_run': { job_id: string };
  'verifier_get_result': { run_id: string; verifier_id: string; status: string; result_asset: { asset_id: string }; logs_asset?: { asset_id: string } };
  'models_list': { models: Array<{ model_id: string; provider: string; label: string; digest: string }> };
  'models_register': { model_id: string; provider: string; label: string; digest: string };
  'models_remove': { removed: boolean };
  'model_roles_get': { tutorial_generation?: string; screen_explainer?: string; anchor_grounding?: string };
  'model_roles_set': { tutorial_generation?: string; screen_explainer?: string; anchor_grounding?: string };
  'ollama_list': { models: string[] };
  'ollama_pull': { job_id: string };
  'ollama_run': { job_id: string };
  'mlx_run': { job_id: string };
  'bench_run': { job_id: string };
  'bench_list': { benches: Array<{ bench_id: string; model_id: string; score: number; created_at: string }> };
  'agent_pipelines_list': { pipelines: string[] };
  'agent_pipeline_run': { job_id: string };
  'agent_pipeline_report': { run_id: string; diagnostics: string[] };
  'exports_list': { exports: Array<{ export_id: string; output_path: string; bundle_hash: string; warnings: Array<{ code: string; message: string }> }> };
  'export_verify_bundle': { valid: boolean; issues: string[] };
  'jobs_list': { jobs: Array<{ job_id: string; job_type: string; session_id?: string; status: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED'; created_at: string; started_at?: string; ended_at?: string; progress?: { stage: string; pct: number; counters: { done: number; total: number } }; error?: { code: string; message: string; details?: string; recoverable: boolean; action_hint?: string } }> };
  'jobs_get': { job_id: string; job_type: string; session_id?: string; status: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED'; created_at: string; started_at?: string; ended_at?: string; progress?: { stage: string; pct: number; counters: { done: number; total: number } }; error?: { code: string; message: string; details?: string; recoverable: boolean; action_hint?: string } };
  'jobs_cancel': { accepted: boolean };
}

export interface GeneratedIpcClient {
  app_get_build_info(payload: IpcRequestMap['app_get_build_info']): Promise<AppResult<IpcResponseMap['app_get_build_info']>>;
  app_get_permissions_status(payload: IpcRequestMap['app_get_permissions_status']): Promise<AppResult<IpcResponseMap['app_get_permissions_status']>>;
  settings_get(payload: IpcRequestMap['settings_get']): Promise<AppResult<IpcResponseMap['settings_get']>>;
  settings_set(payload: IpcRequestMap['settings_set']): Promise<AppResult<IpcResponseMap['settings_set']>>;
  network_allowlist_get(payload: IpcRequestMap['network_allowlist_get']): Promise<AppResult<IpcResponseMap['network_allowlist_get']>>;
  network_allowlist_set(payload: IpcRequestMap['network_allowlist_set']): Promise<AppResult<IpcResponseMap['network_allowlist_set']>>;
  session_create(payload: IpcRequestMap['session_create']): Promise<AppResult<IpcResponseMap['session_create']>>;
  session_list(payload: IpcRequestMap['session_list']): Promise<AppResult<IpcResponseMap['session_list']>>;
  session_get(payload: IpcRequestMap['session_get']): Promise<AppResult<IpcResponseMap['session_get']>>;
  session_close(payload: IpcRequestMap['session_close']): Promise<AppResult<IpcResponseMap['session_close']>>;
  timeline_get_keyframes(payload: IpcRequestMap['timeline_get_keyframes']): Promise<AppResult<IpcResponseMap['timeline_get_keyframes']>>;
  timeline_get_events(payload: IpcRequestMap['timeline_get_events']): Promise<AppResult<IpcResponseMap['timeline_get_events']>>;
  timeline_get_thumbnail(payload: IpcRequestMap['timeline_get_thumbnail']): Promise<AppResult<IpcResponseMap['timeline_get_thumbnail']>>;
  capture_get_config(payload: IpcRequestMap['capture_get_config']): Promise<AppResult<IpcResponseMap['capture_get_config']>>;
  capture_set_config(payload: IpcRequestMap['capture_set_config']): Promise<AppResult<IpcResponseMap['capture_set_config']>>;
  capture_start(payload: IpcRequestMap['capture_start']): Promise<AppResult<IpcResponseMap['capture_start']>>;
  capture_stop(payload: IpcRequestMap['capture_stop']): Promise<AppResult<IpcResponseMap['capture_stop']>>;
  capture_get_status(payload: IpcRequestMap['capture_get_status']): Promise<AppResult<IpcResponseMap['capture_get_status']>>;
  ocr_schedule(payload: IpcRequestMap['ocr_schedule']): Promise<AppResult<IpcResponseMap['ocr_schedule']>>;
  ocr_get_status(payload: IpcRequestMap['ocr_get_status']): Promise<AppResult<IpcResponseMap['ocr_get_status']>>;
  ocr_search(payload: IpcRequestMap['ocr_search']): Promise<AppResult<IpcResponseMap['ocr_search']>>;
  ocr_get_blocks_for_frame(payload: IpcRequestMap['ocr_get_blocks_for_frame']): Promise<AppResult<IpcResponseMap['ocr_get_blocks_for_frame']>>;
  evidence_for_time_range(payload: IpcRequestMap['evidence_for_time_range']): Promise<AppResult<IpcResponseMap['evidence_for_time_range']>>;
  evidence_for_step(payload: IpcRequestMap['evidence_for_step']): Promise<AppResult<IpcResponseMap['evidence_for_step']>>;
  evidence_find_text(payload: IpcRequestMap['evidence_find_text']): Promise<AppResult<IpcResponseMap['evidence_find_text']>>;
  evidence_get_coverage(payload: IpcRequestMap['evidence_get_coverage']): Promise<AppResult<IpcResponseMap['evidence_get_coverage']>>;
  steps_generate_candidates(payload: IpcRequestMap['steps_generate_candidates']): Promise<AppResult<IpcResponseMap['steps_generate_candidates']>>;
  steps_list(payload: IpcRequestMap['steps_list']): Promise<AppResult<IpcResponseMap['steps_list']>>;
  steps_get(payload: IpcRequestMap['steps_get']): Promise<AppResult<IpcResponseMap['steps_get']>>;
  steps_apply_edit(payload: IpcRequestMap['steps_apply_edit']): Promise<AppResult<IpcResponseMap['steps_apply_edit']>>;
  steps_validate(payload: IpcRequestMap['steps_validate']): Promise<AppResult<IpcResponseMap['steps_validate']>>;
  anchors_list_for_step(payload: IpcRequestMap['anchors_list_for_step']): Promise<AppResult<IpcResponseMap['anchors_list_for_step']>>;
  anchors_reacquire(payload: IpcRequestMap['anchors_reacquire']): Promise<AppResult<IpcResponseMap['anchors_reacquire']>>;
  anchors_manual_set(payload: IpcRequestMap['anchors_manual_set']): Promise<AppResult<IpcResponseMap['anchors_manual_set']>>;
  anchors_debug(payload: IpcRequestMap['anchors_debug']): Promise<AppResult<IpcResponseMap['anchors_debug']>>;
  tutorial_generate(payload: IpcRequestMap['tutorial_generate']): Promise<AppResult<IpcResponseMap['tutorial_generate']>>;
  tutorial_export_pack(payload: IpcRequestMap['tutorial_export_pack']): Promise<AppResult<IpcResponseMap['tutorial_export_pack']>>;
  tutorial_validate_export(payload: IpcRequestMap['tutorial_validate_export']): Promise<AppResult<IpcResponseMap['tutorial_validate_export']>>;
  explain_this_screen(payload: IpcRequestMap['explain_this_screen']): Promise<AppResult<IpcResponseMap['explain_this_screen']>>;
  proof_get_view(payload: IpcRequestMap['proof_get_view']): Promise<AppResult<IpcResponseMap['proof_get_view']>>;
  runbook_create(payload: IpcRequestMap['runbook_create']): Promise<AppResult<IpcResponseMap['runbook_create']>>;
  runbook_update(payload: IpcRequestMap['runbook_update']): Promise<AppResult<IpcResponseMap['runbook_update']>>;
  runbook_export(payload: IpcRequestMap['runbook_export']): Promise<AppResult<IpcResponseMap['runbook_export']>>;
  proof_export_bundle(payload: IpcRequestMap['proof_export_bundle']): Promise<AppResult<IpcResponseMap['proof_export_bundle']>>;
  verifier_list(payload: IpcRequestMap['verifier_list']): Promise<AppResult<IpcResponseMap['verifier_list']>>;
  verifier_run(payload: IpcRequestMap['verifier_run']): Promise<AppResult<IpcResponseMap['verifier_run']>>;
  verifier_get_result(payload: IpcRequestMap['verifier_get_result']): Promise<AppResult<IpcResponseMap['verifier_get_result']>>;
  models_list(payload: IpcRequestMap['models_list']): Promise<AppResult<IpcResponseMap['models_list']>>;
  models_register(payload: IpcRequestMap['models_register']): Promise<AppResult<IpcResponseMap['models_register']>>;
  models_remove(payload: IpcRequestMap['models_remove']): Promise<AppResult<IpcResponseMap['models_remove']>>;
  model_roles_get(payload: IpcRequestMap['model_roles_get']): Promise<AppResult<IpcResponseMap['model_roles_get']>>;
  model_roles_set(payload: IpcRequestMap['model_roles_set']): Promise<AppResult<IpcResponseMap['model_roles_set']>>;
  ollama_list(payload: IpcRequestMap['ollama_list']): Promise<AppResult<IpcResponseMap['ollama_list']>>;
  ollama_pull(payload: IpcRequestMap['ollama_pull']): Promise<AppResult<IpcResponseMap['ollama_pull']>>;
  ollama_run(payload: IpcRequestMap['ollama_run']): Promise<AppResult<IpcResponseMap['ollama_run']>>;
  mlx_run(payload: IpcRequestMap['mlx_run']): Promise<AppResult<IpcResponseMap['mlx_run']>>;
  bench_run(payload: IpcRequestMap['bench_run']): Promise<AppResult<IpcResponseMap['bench_run']>>;
  bench_list(payload: IpcRequestMap['bench_list']): Promise<AppResult<IpcResponseMap['bench_list']>>;
  agent_pipelines_list(payload: IpcRequestMap['agent_pipelines_list']): Promise<AppResult<IpcResponseMap['agent_pipelines_list']>>;
  agent_pipeline_run(payload: IpcRequestMap['agent_pipeline_run']): Promise<AppResult<IpcResponseMap['agent_pipeline_run']>>;
  agent_pipeline_report(payload: IpcRequestMap['agent_pipeline_report']): Promise<AppResult<IpcResponseMap['agent_pipeline_report']>>;
  exports_list(payload: IpcRequestMap['exports_list']): Promise<AppResult<IpcResponseMap['exports_list']>>;
  export_verify_bundle(payload: IpcRequestMap['export_verify_bundle']): Promise<AppResult<IpcResponseMap['export_verify_bundle']>>;
  jobs_list(payload: IpcRequestMap['jobs_list']): Promise<AppResult<IpcResponseMap['jobs_list']>>;
  jobs_get(payload: IpcRequestMap['jobs_get']): Promise<AppResult<IpcResponseMap['jobs_get']>>;
  jobs_cancel(payload: IpcRequestMap['jobs_cancel']): Promise<AppResult<IpcResponseMap['jobs_cancel']>>;
}

export function bindGeneratedClient(client: IpcClient): GeneratedIpcClient {
  return {
    app_get_build_info: (payload: IpcRequestMap['app_get_build_info']) => client.invoke<IpcRequestMap['app_get_build_info'], IpcResponseMap['app_get_build_info']>('app_get_build_info', payload),
    app_get_permissions_status: (payload: IpcRequestMap['app_get_permissions_status']) => client.invoke<IpcRequestMap['app_get_permissions_status'], IpcResponseMap['app_get_permissions_status']>('app_get_permissions_status', payload),
    settings_get: (payload: IpcRequestMap['settings_get']) => client.invoke<IpcRequestMap['settings_get'], IpcResponseMap['settings_get']>('settings_get', payload),
    settings_set: (payload: IpcRequestMap['settings_set']) => client.invoke<IpcRequestMap['settings_set'], IpcResponseMap['settings_set']>('settings_set', payload),
    network_allowlist_get: (payload: IpcRequestMap['network_allowlist_get']) => client.invoke<IpcRequestMap['network_allowlist_get'], IpcResponseMap['network_allowlist_get']>('network_allowlist_get', payload),
    network_allowlist_set: (payload: IpcRequestMap['network_allowlist_set']) => client.invoke<IpcRequestMap['network_allowlist_set'], IpcResponseMap['network_allowlist_set']>('network_allowlist_set', payload),
    session_create: (payload: IpcRequestMap['session_create']) => client.invoke<IpcRequestMap['session_create'], IpcResponseMap['session_create']>('session_create', payload),
    session_list: (payload: IpcRequestMap['session_list']) => client.invoke<IpcRequestMap['session_list'], IpcResponseMap['session_list']>('session_list', payload),
    session_get: (payload: IpcRequestMap['session_get']) => client.invoke<IpcRequestMap['session_get'], IpcResponseMap['session_get']>('session_get', payload),
    session_close: (payload: IpcRequestMap['session_close']) => client.invoke<IpcRequestMap['session_close'], IpcResponseMap['session_close']>('session_close', payload),
    timeline_get_keyframes: (payload: IpcRequestMap['timeline_get_keyframes']) => client.invoke<IpcRequestMap['timeline_get_keyframes'], IpcResponseMap['timeline_get_keyframes']>('timeline_get_keyframes', payload),
    timeline_get_events: (payload: IpcRequestMap['timeline_get_events']) => client.invoke<IpcRequestMap['timeline_get_events'], IpcResponseMap['timeline_get_events']>('timeline_get_events', payload),
    timeline_get_thumbnail: (payload: IpcRequestMap['timeline_get_thumbnail']) => client.invoke<IpcRequestMap['timeline_get_thumbnail'], IpcResponseMap['timeline_get_thumbnail']>('timeline_get_thumbnail', payload),
    capture_get_config: (payload: IpcRequestMap['capture_get_config']) => client.invoke<IpcRequestMap['capture_get_config'], IpcResponseMap['capture_get_config']>('capture_get_config', payload),
    capture_set_config: (payload: IpcRequestMap['capture_set_config']) => client.invoke<IpcRequestMap['capture_set_config'], IpcResponseMap['capture_set_config']>('capture_set_config', payload),
    capture_start: (payload: IpcRequestMap['capture_start']) => client.invoke<IpcRequestMap['capture_start'], IpcResponseMap['capture_start']>('capture_start', payload),
    capture_stop: (payload: IpcRequestMap['capture_stop']) => client.invoke<IpcRequestMap['capture_stop'], IpcResponseMap['capture_stop']>('capture_stop', payload),
    capture_get_status: (payload: IpcRequestMap['capture_get_status']) => client.invoke<IpcRequestMap['capture_get_status'], IpcResponseMap['capture_get_status']>('capture_get_status', payload),
    ocr_schedule: (payload: IpcRequestMap['ocr_schedule']) => client.invoke<IpcRequestMap['ocr_schedule'], IpcResponseMap['ocr_schedule']>('ocr_schedule', payload),
    ocr_get_status: (payload: IpcRequestMap['ocr_get_status']) => client.invoke<IpcRequestMap['ocr_get_status'], IpcResponseMap['ocr_get_status']>('ocr_get_status', payload),
    ocr_search: (payload: IpcRequestMap['ocr_search']) => client.invoke<IpcRequestMap['ocr_search'], IpcResponseMap['ocr_search']>('ocr_search', payload),
    ocr_get_blocks_for_frame: (payload: IpcRequestMap['ocr_get_blocks_for_frame']) => client.invoke<IpcRequestMap['ocr_get_blocks_for_frame'], IpcResponseMap['ocr_get_blocks_for_frame']>('ocr_get_blocks_for_frame', payload),
    evidence_for_time_range: (payload: IpcRequestMap['evidence_for_time_range']) => client.invoke<IpcRequestMap['evidence_for_time_range'], IpcResponseMap['evidence_for_time_range']>('evidence_for_time_range', payload),
    evidence_for_step: (payload: IpcRequestMap['evidence_for_step']) => client.invoke<IpcRequestMap['evidence_for_step'], IpcResponseMap['evidence_for_step']>('evidence_for_step', payload),
    evidence_find_text: (payload: IpcRequestMap['evidence_find_text']) => client.invoke<IpcRequestMap['evidence_find_text'], IpcResponseMap['evidence_find_text']>('evidence_find_text', payload),
    evidence_get_coverage: (payload: IpcRequestMap['evidence_get_coverage']) => client.invoke<IpcRequestMap['evidence_get_coverage'], IpcResponseMap['evidence_get_coverage']>('evidence_get_coverage', payload),
    steps_generate_candidates: (payload: IpcRequestMap['steps_generate_candidates']) => client.invoke<IpcRequestMap['steps_generate_candidates'], IpcResponseMap['steps_generate_candidates']>('steps_generate_candidates', payload),
    steps_list: (payload: IpcRequestMap['steps_list']) => client.invoke<IpcRequestMap['steps_list'], IpcResponseMap['steps_list']>('steps_list', payload),
    steps_get: (payload: IpcRequestMap['steps_get']) => client.invoke<IpcRequestMap['steps_get'], IpcResponseMap['steps_get']>('steps_get', payload),
    steps_apply_edit: (payload: IpcRequestMap['steps_apply_edit']) => client.invoke<IpcRequestMap['steps_apply_edit'], IpcResponseMap['steps_apply_edit']>('steps_apply_edit', payload),
    steps_validate: (payload: IpcRequestMap['steps_validate']) => client.invoke<IpcRequestMap['steps_validate'], IpcResponseMap['steps_validate']>('steps_validate', payload),
    anchors_list_for_step: (payload: IpcRequestMap['anchors_list_for_step']) => client.invoke<IpcRequestMap['anchors_list_for_step'], IpcResponseMap['anchors_list_for_step']>('anchors_list_for_step', payload),
    anchors_reacquire: (payload: IpcRequestMap['anchors_reacquire']) => client.invoke<IpcRequestMap['anchors_reacquire'], IpcResponseMap['anchors_reacquire']>('anchors_reacquire', payload),
    anchors_manual_set: (payload: IpcRequestMap['anchors_manual_set']) => client.invoke<IpcRequestMap['anchors_manual_set'], IpcResponseMap['anchors_manual_set']>('anchors_manual_set', payload),
    anchors_debug: (payload: IpcRequestMap['anchors_debug']) => client.invoke<IpcRequestMap['anchors_debug'], IpcResponseMap['anchors_debug']>('anchors_debug', payload),
    tutorial_generate: (payload: IpcRequestMap['tutorial_generate']) => client.invoke<IpcRequestMap['tutorial_generate'], IpcResponseMap['tutorial_generate']>('tutorial_generate', payload),
    tutorial_export_pack: (payload: IpcRequestMap['tutorial_export_pack']) => client.invoke<IpcRequestMap['tutorial_export_pack'], IpcResponseMap['tutorial_export_pack']>('tutorial_export_pack', payload),
    tutorial_validate_export: (payload: IpcRequestMap['tutorial_validate_export']) => client.invoke<IpcRequestMap['tutorial_validate_export'], IpcResponseMap['tutorial_validate_export']>('tutorial_validate_export', payload),
    explain_this_screen: (payload: IpcRequestMap['explain_this_screen']) => client.invoke<IpcRequestMap['explain_this_screen'], IpcResponseMap['explain_this_screen']>('explain_this_screen', payload),
    proof_get_view: (payload: IpcRequestMap['proof_get_view']) => client.invoke<IpcRequestMap['proof_get_view'], IpcResponseMap['proof_get_view']>('proof_get_view', payload),
    runbook_create: (payload: IpcRequestMap['runbook_create']) => client.invoke<IpcRequestMap['runbook_create'], IpcResponseMap['runbook_create']>('runbook_create', payload),
    runbook_update: (payload: IpcRequestMap['runbook_update']) => client.invoke<IpcRequestMap['runbook_update'], IpcResponseMap['runbook_update']>('runbook_update', payload),
    runbook_export: (payload: IpcRequestMap['runbook_export']) => client.invoke<IpcRequestMap['runbook_export'], IpcResponseMap['runbook_export']>('runbook_export', payload),
    proof_export_bundle: (payload: IpcRequestMap['proof_export_bundle']) => client.invoke<IpcRequestMap['proof_export_bundle'], IpcResponseMap['proof_export_bundle']>('proof_export_bundle', payload),
    verifier_list: (payload: IpcRequestMap['verifier_list']) => client.invoke<IpcRequestMap['verifier_list'], IpcResponseMap['verifier_list']>('verifier_list', payload),
    verifier_run: (payload: IpcRequestMap['verifier_run']) => client.invoke<IpcRequestMap['verifier_run'], IpcResponseMap['verifier_run']>('verifier_run', payload),
    verifier_get_result: (payload: IpcRequestMap['verifier_get_result']) => client.invoke<IpcRequestMap['verifier_get_result'], IpcResponseMap['verifier_get_result']>('verifier_get_result', payload),
    models_list: (payload: IpcRequestMap['models_list']) => client.invoke<IpcRequestMap['models_list'], IpcResponseMap['models_list']>('models_list', payload),
    models_register: (payload: IpcRequestMap['models_register']) => client.invoke<IpcRequestMap['models_register'], IpcResponseMap['models_register']>('models_register', payload),
    models_remove: (payload: IpcRequestMap['models_remove']) => client.invoke<IpcRequestMap['models_remove'], IpcResponseMap['models_remove']>('models_remove', payload),
    model_roles_get: (payload: IpcRequestMap['model_roles_get']) => client.invoke<IpcRequestMap['model_roles_get'], IpcResponseMap['model_roles_get']>('model_roles_get', payload),
    model_roles_set: (payload: IpcRequestMap['model_roles_set']) => client.invoke<IpcRequestMap['model_roles_set'], IpcResponseMap['model_roles_set']>('model_roles_set', payload),
    ollama_list: (payload: IpcRequestMap['ollama_list']) => client.invoke<IpcRequestMap['ollama_list'], IpcResponseMap['ollama_list']>('ollama_list', payload),
    ollama_pull: (payload: IpcRequestMap['ollama_pull']) => client.invoke<IpcRequestMap['ollama_pull'], IpcResponseMap['ollama_pull']>('ollama_pull', payload),
    ollama_run: (payload: IpcRequestMap['ollama_run']) => client.invoke<IpcRequestMap['ollama_run'], IpcResponseMap['ollama_run']>('ollama_run', payload),
    mlx_run: (payload: IpcRequestMap['mlx_run']) => client.invoke<IpcRequestMap['mlx_run'], IpcResponseMap['mlx_run']>('mlx_run', payload),
    bench_run: (payload: IpcRequestMap['bench_run']) => client.invoke<IpcRequestMap['bench_run'], IpcResponseMap['bench_run']>('bench_run', payload),
    bench_list: (payload: IpcRequestMap['bench_list']) => client.invoke<IpcRequestMap['bench_list'], IpcResponseMap['bench_list']>('bench_list', payload),
    agent_pipelines_list: (payload: IpcRequestMap['agent_pipelines_list']) => client.invoke<IpcRequestMap['agent_pipelines_list'], IpcResponseMap['agent_pipelines_list']>('agent_pipelines_list', payload),
    agent_pipeline_run: (payload: IpcRequestMap['agent_pipeline_run']) => client.invoke<IpcRequestMap['agent_pipeline_run'], IpcResponseMap['agent_pipeline_run']>('agent_pipeline_run', payload),
    agent_pipeline_report: (payload: IpcRequestMap['agent_pipeline_report']) => client.invoke<IpcRequestMap['agent_pipeline_report'], IpcResponseMap['agent_pipeline_report']>('agent_pipeline_report', payload),
    exports_list: (payload: IpcRequestMap['exports_list']) => client.invoke<IpcRequestMap['exports_list'], IpcResponseMap['exports_list']>('exports_list', payload),
    export_verify_bundle: (payload: IpcRequestMap['export_verify_bundle']) => client.invoke<IpcRequestMap['export_verify_bundle'], IpcResponseMap['export_verify_bundle']>('export_verify_bundle', payload),
    jobs_list: (payload: IpcRequestMap['jobs_list']) => client.invoke<IpcRequestMap['jobs_list'], IpcResponseMap['jobs_list']>('jobs_list', payload),
    jobs_get: (payload: IpcRequestMap['jobs_get']) => client.invoke<IpcRequestMap['jobs_get'], IpcResponseMap['jobs_get']>('jobs_get', payload),
    jobs_cancel: (payload: IpcRequestMap['jobs_cancel']) => client.invoke<IpcRequestMap['jobs_cancel'], IpcResponseMap['jobs_cancel']>('jobs_cancel', payload),
  };
}
