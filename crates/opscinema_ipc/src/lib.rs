use opscinema_types::IpcCommand;

pub fn locked_command_names() -> Vec<&'static str> {
    IpcCommand::LOCKED_COMMANDS
        .iter()
        .map(|c| c.as_str())
        .collect()
}

pub fn generate_typescript_client() -> String {
    let mut out = String::from("// GENERATED FILE - DO NOT EDIT\n\n");
    out.push_str("export type JsonObject = { [key: string]: JsonValue };\n");
    out.push_str(
        "export type JsonValue = string | number | boolean | null | JsonObject | JsonValue[];\n\n",
    );
    out.push_str(
        "export type AppResult<T> = { ok: true; value: T } | { ok: false; error: AppError };\n",
    );
    out.push_str("export type AppErrorCode =\n");
    out.push_str("  | 'PERMISSION_DENIED'\n  | 'VALIDATION_FAILED'\n  | 'NOT_FOUND'\n  | 'CONFLICT'\n  | 'POLICY_BLOCKED'\n  | 'NETWORK_BLOCKED'\n  | 'EXPORT_GATE_FAILED'\n  | 'PROVIDER_SCHEMA_INVALID'\n  | 'IO'\n  | 'DB'\n  | 'JOB_CANCELLED'\n  | 'UNSUPPORTED'\n  | 'INTERNAL';\n");
    out.push_str("export interface AppError { code: AppErrorCode; message: string; details?: string; recoverable: boolean; action_hint?: string; }\n\n");
    out.push_str("export type IpcCommand =\n");
    for (idx, cmd) in locked_command_names().iter().enumerate() {
        let suffix = if idx + 1 == IpcCommand::LOCKED_COMMANDS.len() {
            ";"
        } else {
            " |"
        };
        out.push_str(&format!("  '{}'{}\n", cmd, suffix));
    }
    out.push_str(
        "\nexport interface IpcClient {\n  invoke<TReq, TRes>(command: IpcCommand, payload: TReq): Promise<AppResult<TRes>>;\n}\n",
    );
    out.push_str("\nexport interface IpcRequestMap {\n");
    for cmd in locked_command_names() {
        let (req, _res) = command_types(cmd);
        out.push_str(&format!("  '{}': {};\n", cmd, req));
    }
    out.push_str("}\n");
    out.push_str("\nexport interface IpcResponseMap {\n");
    for cmd in locked_command_names() {
        let (_req, res) = command_types(cmd);
        out.push_str(&format!("  '{}': {};\n", cmd, res));
    }
    out.push_str("}\n");
    out.push_str("\nexport interface GeneratedIpcClient {\n");
    for cmd in locked_command_names() {
        out.push_str(&format!(
            "  {}(payload: IpcRequestMap['{}']): Promise<AppResult<IpcResponseMap['{}']>>;\n",
            cmd, cmd, cmd
        ));
    }
    out.push_str("}\n");
    out.push_str(
        "\nexport function bindGeneratedClient(client: IpcClient): GeneratedIpcClient {\n",
    );
    out.push_str("  return {\n");
    for cmd in locked_command_names() {
        out.push_str(&format!(
            "    {}: (payload: IpcRequestMap['{}']) => client.invoke<IpcRequestMap['{}'], IpcResponseMap['{}']>('{}', payload),\n",
            cmd, cmd, cmd, cmd, cmd
        ));
    }
    out.push_str("  };\n");
    out.push_str("}\n");
    out
}

fn command_types(command: &str) -> (&'static str, &'static str) {
    match command {
        "app_get_build_info" => (
            "Record<string, never>",
            "{ app_name: string; app_version: string; commit: string; built_at: string }",
        ),
        "app_get_permissions_status" => (
            "Record<string, never>",
            "{ screen_recording: boolean; accessibility: boolean; full_disk_access: boolean }",
        ),
        "settings_get" => (
            "Record<string, never>",
            "{ offline_mode: boolean; allow_input_capture: boolean; allow_window_metadata: boolean }",
        ),
        "settings_set" => (
            "{ offline_mode: boolean; allow_input_capture: boolean; allow_window_metadata: boolean }",
            "{ offline_mode: boolean; allow_input_capture: boolean; allow_window_metadata: boolean }",
        ),
        "network_allowlist_get" => ("Record<string, never>", "{ entries: string[] }"),
        "network_allowlist_set" => ("{ entries: string[] }", "{ entries: string[] }"),
        "session_create" => (
            "{ label: string; metadata: Record<string, string> }",
            "{ session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string }",
        ),
        "session_list" => (
            "{ limit?: number }",
            "Array<{ session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string }>",
        ),
        "session_get" => (
            "{ session_id: string }",
            "{ summary: { session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string }; metadata: Record<string, string> }",
        ),
        "session_close" => (
            "{ session_id: string }",
            "{ session_id: string; label: string; created_at: string; closed_at?: string; head_seq: number; head_hash: string }",
        ),
        "timeline_get_keyframes" => (
            "{ session_id: string; start_ms: number; end_ms: number }",
            "{ keyframes: Array<{ frame_event_id: string; frame_ms: number; asset: { asset_id: string } }> }",
        ),
        "timeline_get_events" => (
            "{ session_id: string; after_seq?: number; limit?: number }",
            "{ events: Array<{ seq: number; event_id: string; event_type: string; frame_ms?: number }> }",
        ),
        "timeline_get_thumbnail" => ("{ session_id: string; frame_event_id: string }", "{ asset_id: string }"),
        "capture_get_config" => (
            "Record<string, never>",
            "{ keyframe_interval_ms: number; include_input: boolean; include_window_meta: boolean }",
        ),
        "capture_set_config" => (
            "{ keyframe_interval_ms: number; include_input: boolean; include_window_meta: boolean }",
            "{ keyframe_interval_ms: number; include_input: boolean; include_window_meta: boolean }",
        ),
        "capture_start" | "capture_stop" => (
            "{ session_id: string }",
            "{ state: 'IDLE' | 'CAPTURING' | 'STOPPED'; session_id?: string; started_at?: string }",
        ),
        "capture_get_status" => (
            "{ session_id?: string }",
            "{ state: 'IDLE' | 'CAPTURING' | 'STOPPED'; session_id?: string; started_at?: string }",
        ),
        "ocr_schedule" => (
            "{ session_id: string; start_ms?: number; end_ms?: number }",
            "{ job_id: string }",
        ),
        "ocr_get_status" => (
            "{ session_id: string }",
            "{ queued_frames: number; indexed_frames: number }",
        ),
        "ocr_search" => (
            "{ session_id: string; query: string }",
            "{ hits: Array<{ frame_ms: number; block_id: string; snippet: string }> }",
        ),
        "ocr_get_blocks_for_frame" => (
            "{ session_id: string; frame_event_id: string }",
            "{ blocks: Array<{ ocr_block_id: string; bbox_norm: { x: number; y: number; w: number; h: number }; text: string; confidence: number; language?: string }> }",
        ),
        "evidence_for_step" => (
            "{ session_id: string; step_id: string }",
            "{ evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> }",
        ),
        "evidence_find_text" => (
            "{ session_id: string; query: string }",
            "{ evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> }",
        ),
        "steps_generate_candidates" => ("{ session_id: string }", "{ job_id: string }"),
        "steps_list" => (
            "{ session_id: string }",
            "{ steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }>; head_seq: number }",
        ),
        "steps_get" => (
            "{ session_id: string; step_id: string }",
            "{ step: { step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }; anchors: Array<{ anchor_id: string; step_id: string; kind: string; target_signature: string; confidence: number; degraded: boolean; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> }",
        ),
        "steps_apply_edit" => (
            "{ session_id: string; base_seq: number; op: JsonObject }",
            "{ head_seq: number; applied: boolean }",
        ),
        "steps_validate" => (
            "{ session_id: string }",
            "{ schema_valid: boolean; evidence_valid: boolean; errors: string[] }",
        ),
        "anchors_list_for_step" => (
            "{ session_id: string; step_id: string }",
            "{ anchors: Array<{ anchor_id: string; step_id: string; kind: string; target_signature: string; confidence: number; degraded: boolean; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> }",
        ),
        "anchors_reacquire" => ("{ session_id: string; step_id: string }", "{ job_id: string }"),
        "anchors_manual_set" => (
            "{ session_id: string; anchor_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }>; note?: string }",
            "{ anchor: { anchor_id: string; step_id: string; kind: string; target_signature: string; confidence: number; degraded: boolean; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> } }",
        ),
        "anchors_debug" => (
            "{ session_id: string; step_id: string }",
            "{ checks: string[] }",
        ),
        "tutorial_generate" => ("{ session_id: string }", "{ job_id: string }"),
        "tutorial_validate_export" => (
            "{ session_id: string }",
            "{ allowed: boolean; reasons: string[] }",
        ),
        "tutorial_export_pack" => (
            "{ session_id: string; output_dir: string }",
            "{ export_id: string; output_path: string; bundle_hash: string; warnings: Array<{ code: string; message: string }> }",
        ),
        "explain_this_screen" => (
            "{ session_id: string; frame_event_id: string }",
            "{ job_id: string }",
        ),
        "evidence_get_coverage" => (
            "{ session_id: string }",
            "{ missing_step_ids: string[]; missing_generated_block_ids: string[]; pass: boolean }",
        ),
        "evidence_for_time_range" => (
            "{ session_id: string; start_ms: number; end_ms: number }",
            "{ evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> }",
        ),
        "proof_get_view" => (
            "{ session_id: string }",
            "{ steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }>; evidence: { evidence: Array<{ evidence_id: string; kind: string; source_id: string; locators: Array<{ locator_type: string; asset_id?: string; frame_ms?: number; bbox_norm?: { x: number; y: number; w: number; h: number }; text_offset?: { start: number; end: number }; note?: string }> }> }; warnings: Array<{ code: string; message: string }> }",
        ),
        "runbook_create" => (
            "{ session_id: string; title: string }",
            "{ runbook_id: string; title: string; steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }> }",
        ),
        "runbook_update" => (
            "{ runbook_id: string; title?: string }",
            "{ runbook_id: string; title: string; steps: Array<{ step_id: string; title: string; order_index: number; body: { blocks: Array<{ block_id: string; text: string; provenance: 'human' | 'generated'; evidence_refs: string[] }> }; risk_tags: string[]; branch_label?: string }> }",
        ),
        "runbook_export" | "proof_export_bundle" => (
            "{ runbook_id?: string; session_id?: string; output_dir: string }",
            "{ export_id: string; output_path: string; bundle_hash: string; warnings: Array<{ code: string; message: string }> }",
        ),
        "verifier_list" => (
            "{ include_disabled: boolean }",
            "{ verifiers: Array<{ verifier_id: string; kind: string; timeout_secs: number; command_allowlist: string[] }> }",
        ),
        "verifier_run" => ("{ session_id: string; verifier_id: string }", "{ job_id: string }"),
        "verifier_get_result" => (
            "{ run_id: string }",
            "{ run_id: string; verifier_id: string; status: string; result_asset: { asset_id: string }; logs_asset?: { asset_id: string } }",
        ),
        "models_list" => (
            "{ include_unhealthy: boolean }",
            "{ models: Array<{ model_id: string; provider: string; label: string; digest: string }> }",
        ),
        "models_register" => (
            "{ provider: string; label: string; model_name: string; digest: string }",
            "{ model_id: string; provider: string; label: string; digest: string }",
        ),
        "models_remove" => ("{ model_id: string }", "{ removed: boolean }"),
        "model_roles_set" => (
            "{ tutorial_generation?: string; screen_explainer?: string; anchor_grounding?: string }",
            "{ tutorial_generation?: string; screen_explainer?: string; anchor_grounding?: string }",
        ),
        "ollama_list" => ("{ host?: string }", "{ models: string[] }"),
        "ollama_pull" => ("{ host?: string; model: string }", "{ job_id: string }"),
        "ollama_run" => ("{ model_id: string; prompt: string }", "{ job_id: string }"),
        "mlx_run" => ("{ model_id: string; prompt: string }", "{ job_id: string }"),
        "bench_run" => ("{ model_id: string; benchmark: string }", "{ job_id: string }"),
        "bench_list" => (
            "{ limit?: number }",
            "{ benches: Array<{ bench_id: string; model_id: string; score: number; created_at: string }> }",
        ),
        "model_roles_get" => (
            "Record<string, never>",
            "{ tutorial_generation?: string; screen_explainer?: string; anchor_grounding?: string }",
        ),
        "agent_pipelines_list" => (
            "Record<string, never>",
            "{ pipelines: string[] }",
        ),
        "agent_pipeline_run" => (
            "{ session_id: string; pipeline_id: string }",
            "{ job_id: string }",
        ),
        "agent_pipeline_report" => (
            "{ run_id: string }",
            "{ run_id: string; diagnostics: string[] }",
        ),
        "exports_list" => (
            "{ session_id?: string }",
            "{ exports: Array<{ export_id: string; output_path: string; bundle_hash: string; warnings: Array<{ code: string; message: string }> }> }",
        ),
        "export_verify_bundle" => (
            "{ bundle_path: string }",
            "{ valid: boolean; issues: string[] }",
        ),
        "jobs_list" => (
            "{ session_id?: string; status?: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED' }",
            "{ jobs: Array<{ job_id: string; job_type: string; session_id?: string; status: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED'; created_at: string; started_at?: string; ended_at?: string; progress?: { stage: string; pct: number; counters: { done: number; total: number } }; error?: { code: string; message: string; details?: string; recoverable: boolean; action_hint?: string } }> }",
        ),
        "jobs_get" => (
            "{ job_id: string }",
            "{ job_id: string; job_type: string; session_id?: string; status: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELLED'; created_at: string; started_at?: string; ended_at?: string; progress?: { stage: string; pct: number; counters: { done: number; total: number } }; error?: { code: string; message: string; details?: string; recoverable: boolean; action_hint?: string } }",
        ),
        "jobs_cancel" => ("{ job_id: string }", "{ accepted: boolean }"),
        _ => ("JsonObject", "JsonObject"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn generated_client_has_no_any_and_is_deterministic() {
        let ts = generate_typescript_client();
        assert!(!ts.contains(" any"));
        assert!(!ts.contains(": any"));

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/desktop/ui/src/ipc/generated.ts");
        fs::write(&path, ts.as_bytes()).expect("write generated ts");

        let on_disk = fs::read_to_string(path).expect("read generated ts");
        assert_eq!(ts, on_disk);
    }

    #[test]
    fn command_list_is_locked_size() {
        assert_eq!(IpcCommand::LOCKED_COMMANDS.len(), 66);
    }
}
