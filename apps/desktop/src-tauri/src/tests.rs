use crate::api;
use crate::capture::coord::{normalize_bbox, normalize_point, RawPoint};
use crate::exports::{tutorial_pack, verify};
use crate::policy::export_gate::{
    ensure_generated_blocks_have_evidence, tutorial_pack_gate, ExportGateInput,
};
use crate::storage::{asset_store, db::Storage, event_store, gc};
use crate::util::canon_json::to_canonical_json;
use opscinema_export_manifest::{BundleType, ExportManifestV1, PolicyAttestations};
use opscinema_ipc::generate_typescript_client;
use opscinema_types::{
    AppErrorCode, CaptureStartRequest, CaptureStatusEvent, EventStreamEnvelope, JobCounters,
    JobProgressEvent, JobStatus, JobStatusEvent, SessionCreateRequest, Step, StructuredText,
    TextBlock, TextBlockProvenance,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn assert_or_update_hash(expected_file: &Path, actual_hash: &str, label: &str) {
    let accept = std::env::var("OPSCINEMA_ACCEPT_FIXTURE_HASH")
        .map(|v| v == "1")
        .unwrap_or(false);
    if accept {
        std::fs::write(expected_file, format!("{actual_hash}\n"))
            .unwrap_or_else(|_| panic!("write expected hash {}", expected_file.display()));
        return;
    }
    let expected = std::fs::read_to_string(expected_file)
        .unwrap_or_else(|_| panic!("read expected hash {}", expected_file.display()));
    assert_eq!(actual_hash, expected.trim(), "{label} hash mismatch");
}

#[test]
fn phase0_ipc_typescript_has_no_any() {
    let ts = generate_typescript_client();
    assert!(!ts.contains(" any"));
    assert!(!ts.contains(": any"));
    assert!(!ts.contains("unknown"));
}

#[test]
fn phase0_error_envelope_permission_denied_path() {
    let err = crate::policy::permissions::require_screen_permission(
        &opscinema_types::PermissionsStatus {
            screen_recording: false,
            accessibility: true,
            full_disk_access: false,
        },
    )
    .expect_err("should fail");
    assert_eq!(err.code, AppErrorCode::PermissionDenied);
    assert!(err.action_hint.is_some());
}

#[test]
fn phase0_capture_permission_denied_has_action_hint() {
    let _env_guard = env_lock();
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "0");
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "permission-denied".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let err = api::capture::capture_start(
        &backend,
        CaptureStartRequest {
            session_id: session.session_id,
        },
    )
    .expect_err("capture must fail without permission");
    assert_eq!(err.code, AppErrorCode::PermissionDenied);
    assert!(err.action_hint.is_some());
    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
}

#[test]
fn phase0_runtime_smoke_build_info_and_session_roundtrip() {
    let build = crate::api::app::app_get_build_info().expect("build info");
    assert!(!build.app_name.is_empty());
    assert!(!build.app_version.is_empty());
    assert!(!build.commit.is_empty());
    assert!(build.built_at.timestamp() >= 0);

    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let created = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "phase0-smoke".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("create session");
    let detail = api::sessions::session_get(
        &backend,
        opscinema_types::SessionGetRequest {
            session_id: created.session_id,
        },
    )
    .expect("get session");
    assert_eq!(detail.summary.session_id, created.session_id);
}

#[test]
fn phase0_event_stream_envelope_schema_fields_present() {
    let job_progress = EventStreamEnvelope {
        stream_seq: 1,
        sent_at: chrono::Utc::now(),
        payload: JobProgressEvent {
            job_id: Uuid::new_v4(),
            stage: "queued".to_string(),
            pct: 0,
            counters: JobCounters { done: 0, total: 1 },
        },
    };
    let job_status = EventStreamEnvelope {
        stream_seq: 2,
        sent_at: chrono::Utc::now(),
        payload: JobStatusEvent {
            job_id: Uuid::new_v4(),
            status: JobStatus::Queued,
        },
    };
    let capture_status = EventStreamEnvelope {
        stream_seq: 3,
        sent_at: chrono::Utc::now(),
        payload: CaptureStatusEvent {
            state: opscinema_types::CaptureState::Idle,
            session_id: None,
        },
    };

    for value in [
        serde_json::to_value(job_progress).expect("job_progress"),
        serde_json::to_value(job_status).expect("job_status"),
        serde_json::to_value(capture_status).expect("capture_status"),
    ] {
        assert!(value.get("stream_seq").is_some());
        assert!(value.get("sent_at").is_some());
        assert!(value.get("payload").is_some());
    }
}

#[cfg(feature = "runtime")]
#[test]
fn phase0_runtime_event_stream_seq_is_monotonic() {
    let bus = crate::api::runtime_events::RuntimeEventBus::default();
    assert_eq!(bus.next_job_status_seq_for_test(), 1);
    assert_eq!(bus.next_job_status_seq_for_test(), 2);
    assert_eq!(bus.next_job_progress_seq_for_test(), 1);
    assert_eq!(bus.next_job_progress_seq_for_test(), 2);
    assert_eq!(bus.next_capture_status_seq_for_test(), 1);
}

#[test]
fn phase0_tauri_command_wrappers_cover_locked_command_list() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api/tauri_commands.rs");
    let content = std::fs::read_to_string(path).expect("read tauri command wrappers");
    for command in opscinema_types::IpcCommand::LOCKED_COMMANDS {
        let signature = format!("pub fn {}", command.as_str());
        assert!(
            content.contains(&signature),
            "missing tauri command wrapper for {}",
            command.as_str()
        );
    }
}

#[test]
fn phase1_append_only_event_hash_chain_and_crash_safety() {
    let storage = Storage::open_in_memory().expect("storage");
    let mut conn = storage.conn().expect("conn");
    let session_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO sessions(session_id,label,created_at,head_seq,head_hash) VALUES (?1,'s',?2,0,'')",
        rusqlite::params![session_id.to_string(), crate::util::time::now_utc_iso()],
    )
    .expect("insert session");

    let store = asset_store::AssetStore::new(storage.assets_root.clone());
    let crash = store.put(
        &conn,
        b"orphan",
        Some(asset_store::CrashPoint::AfterAssetWriteBeforeDb),
    );
    assert!(crash.is_err());

    let (_id1, seq1, h1) = event_store::append_event(
        &mut conn,
        session_id,
        "TestEvent",
        &serde_json::json!({"a":1}),
        None,
    )
    .expect("append 1");
    let (_id2, seq2, h2) = event_store::append_event(
        &mut conn,
        session_id,
        "TestEvent",
        &serde_json::json!({"a":2}),
        None,
    )
    .expect("append 2");

    assert_eq!(seq1, 1);
    assert_eq!(seq2, 2);
    assert_ne!(h1, h2);

    let crash2 = event_store::append_event(
        &mut conn,
        session_id,
        "CrashEvent",
        &serde_json::json!({"x":1}),
        Some(event_store::CrashPoint::AfterEventInsertBeforeCommit),
    );
    assert!(crash2.is_err());

    let events = event_store::query_events(&conn, session_id, None, 100).expect("query");
    assert_eq!(events.len(), 2);
    event_store::validate_hash_chain(&conn, session_id).expect("hash chain");
}

#[test]
fn phase1_file_backed_restart_invariants() {
    let root = tempfile::tempdir().expect("tmp");
    let db_path = root.path().join("state.sqlite");
    let assets = root.path().join("assets");

    let storage = Storage::open(&db_path, &assets).expect("open");
    let mut conn = storage.conn().expect("conn");
    let session_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO sessions(session_id,label,created_at,head_seq,head_hash) VALUES (?1,'s',?2,0,'')",
        rusqlite::params![session_id.to_string(), crate::util::time::now_utc_iso()],
    )
    .expect("insert session");

    let store = asset_store::AssetStore::new(storage.assets_root.clone());
    let _ = store.put(
        &conn,
        b"orphan-on-disk",
        Some(asset_store::CrashPoint::AfterAssetWriteBeforeDb),
    );
    let _ = event_store::append_event(
        &mut conn,
        session_id,
        "CrashEvent",
        &serde_json::json!({"x":1}),
        Some(event_store::CrashPoint::AfterEventInsertBeforeCommit),
    );
    drop(conn);

    let reopened = Storage::open(&db_path, &assets).expect("reopen");
    let conn2 = reopened.conn().expect("conn2");
    let events = event_store::query_events(&conn2, session_id, None, 100).expect("events");
    assert!(events.is_empty(), "crashed event must not commit");
    event_store::validate_hash_chain(&conn2, session_id).expect("hash chain after restart");
    let report = gc::gc_orphan_assets(
        &conn2,
        &asset_store::AssetStore::new(reopened.assets_root.clone()),
        true,
    )
    .expect("gc dry");
    assert_eq!(
        report.orphan_count, 0,
        "orphan rows should not be committed"
    );
    let mut file_count = 0usize;
    for entry in walkdir::WalkDir::new(reopened.assets_root) {
        let entry = entry.expect("walk");
        if entry.file_type().is_file() {
            file_count += 1;
        }
    }
    assert!(
        file_count >= 1,
        "expected on-disk orphaned file after asset-write crash"
    );
}

#[test]
fn phase1_event_references_existing_asset_row() {
    let _env_guard = env_lock();
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "1");
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "asset-ordering".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::capture::capture_start(
        &backend,
        CaptureStartRequest {
            session_id: session.session_id,
        },
    )
    .expect("capture");

    let conn = backend.storage.conn().expect("conn");
    let events = event_store::query_events(&conn, session.session_id, None, 100).expect("events");
    let keyframe = events
        .into_iter()
        .find(|e| e.event_type == "KeyframeCaptured")
        .expect("keyframe event");
    let asset_id = serde_json::from_str::<serde_json::Value>(&keyframe.payload_canon_json)
        .expect("json")
        .get("asset_id")
        .and_then(|v| v.as_str())
        .expect("asset id")
        .to_string();
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM assets WHERE asset_id=?1",
            rusqlite::params![asset_id],
            |r| r.get(0),
        )
        .expect("asset count");
    assert_eq!(exists, 1);
    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
}

#[test]
fn phase1_ocr_event_references_existing_asset_row() {
    let _env_guard = env_lock();
    std::env::remove_var("OPSCINEMA_VISION_RAW_JSON");
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "ocr-asset-ordering".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::ocr::ocr_schedule(
        &backend,
        opscinema_types::OcrScheduleRequest {
            session_id: session.session_id,
            start_ms: Some(0),
            end_ms: Some(0),
        },
    )
    .expect("ocr");

    let conn = backend.storage.conn().expect("conn");
    let events = event_store::query_events(&conn, session.session_id, None, 200).expect("events");
    let ocr_event = events
        .into_iter()
        .find(|e| e.event_type == "OcrBlocksPersisted")
        .expect("ocr event");
    let asset_id = serde_json::from_str::<serde_json::Value>(&ocr_event.payload_canon_json)
        .expect("json")
        .get("ocr_asset_id")
        .and_then(|v| v.as_str())
        .expect("ocr asset id")
        .to_string();
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM assets WHERE asset_id=?1",
            rusqlite::params![asset_id],
            |r| r.get(0),
        )
        .expect("asset count");
    assert_eq!(exists, 1);
    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
}

#[test]
fn phase2_coord_normalization() {
    let p = normalize_point(RawPoint { x: 960.0, y: 540.0 }, 1920.0, 1080.0);
    assert_eq!(p, (5000, 5000));
    let b = normalize_bbox(0.0, 0.0, 1920.0, 1080.0, 1920.0, 1080.0);
    assert_eq!(b.w, 10000);
    assert_eq!(b.h, 10000);
}

#[derive(Debug, Deserialize)]
struct CoordFixture {
    width: f64,
    height: f64,
    point: CoordFixturePoint,
    expected: CoordFixtureExpected,
}

#[derive(Debug, Deserialize)]
struct CoordFixturePoint {
    x: f64,
    y: f64,
}

#[derive(Debug, Deserialize)]
struct CoordFixtureExpected {
    x: u32,
    y: u32,
}

#[test]
fn phase2_coord_normalization_fixtures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/fixtures/capture");
    for fixture_name in ["single_monitor_retina.json", "multi_monitor_secondary.json"] {
        let raw = std::fs::read_to_string(root.join(fixture_name)).expect("fixture");
        let fixture: CoordFixture = serde_json::from_str(&raw).expect("json");
        let (x, y) = normalize_point(
            RawPoint {
                x: fixture.point.x,
                y: fixture.point.y,
            },
            fixture.width,
            fixture.height,
        );
        assert_eq!(x, fixture.expected.x, "{fixture_name} x");
        assert_eq!(y, fixture.expected.y, "{fixture_name} y");
    }
}

#[test]
fn phase2_capture_status_transitions_and_multi_keyframes() {
    let _env_guard = env_lock();
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "1");
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    std::env::set_var("OPSCINEMA_CAPTURE_BURST_FRAMES", "3");

    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "capture-loop".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");

    let _ = api::capture::capture_set_config(opscinema_types::CaptureConfig {
        keyframe_interval_ms: 120,
        include_input: false,
        include_window_meta: false,
    })
    .expect("set config");
    let status_started = api::capture::capture_start(
        &backend,
        CaptureStartRequest {
            session_id: session.session_id,
        },
    )
    .expect("capture start");
    assert_eq!(
        status_started.state,
        opscinema_types::CaptureState::Capturing
    );
    std::thread::sleep(std::time::Duration::from_millis(380));

    let keyframes = api::timeline::timeline_get_keyframes(
        &backend,
        opscinema_types::TimelineKeyframesRequest {
            session_id: session.session_id,
            start_ms: 0,
            end_ms: i64::MAX,
        },
    )
    .expect("timeline");
    assert!(
        keyframes.keyframes.len() >= 2,
        "expected at least 2 keyframes, got {}",
        keyframes.keyframes.len()
    );

    let status_stopped = api::capture::capture_stop(
        &backend,
        opscinema_types::CaptureStopRequest {
            session_id: session.session_id,
        },
    )
    .expect("capture stop");
    assert_eq!(status_stopped.state, opscinema_types::CaptureState::Stopped);

    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
    std::env::remove_var("OPSCINEMA_CAPTURE_BURST_FRAMES");
}

#[test]
fn phase2_capture_rejects_concurrent_session_start() {
    let _env_guard = env_lock();
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "1");
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    std::env::set_var("OPSCINEMA_CAPTURE_BURST_FRAMES", "1");

    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session_a = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "capture-a".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session a");
    let session_b = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "capture-b".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session b");

    let _ = api::capture::capture_start(
        &backend,
        CaptureStartRequest {
            session_id: session_a.session_id,
        },
    )
    .expect("capture a start");
    let err = api::capture::capture_start(
        &backend,
        CaptureStartRequest {
            session_id: session_b.session_id,
        },
    )
    .expect_err("capture b should conflict");
    assert_eq!(err.code, AppErrorCode::Conflict);

    let _ = api::capture::capture_stop(
        &backend,
        opscinema_types::CaptureStopRequest {
            session_id: session_a.session_id,
        },
    )
    .expect("stop");

    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
    std::env::remove_var("OPSCINEMA_CAPTURE_BURST_FRAMES");
}

#[test]
fn phase3_ocr_provider_schema_invalid_is_hard_fail() {
    let _env_guard = env_lock();
    std::env::set_var(
        "OPSCINEMA_VISION_RAW_JSON",
        r#"[{"text":"ok","confidence":1.2,"x":0.1,"y":0.1,"w":0.2,"h":0.2}]"#,
    );
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "bad-ocr".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let err = api::ocr::ocr_schedule(
        &backend,
        opscinema_types::OcrScheduleRequest {
            session_id: session.session_id,
            start_ms: Some(0),
            end_ms: Some(0),
        },
    )
    .expect_err("must fail schema");
    assert_eq!(err.code, AppErrorCode::ProviderSchemaInvalid);
    std::env::remove_var("OPSCINEMA_VISION_RAW_JSON");
}

#[test]
fn phase4_evidence_gate_blocks_generated_without_refs() {
    let step = Step {
        step_id: Uuid::new_v4(),
        order_index: 0,
        title: "t".to_string(),
        body: StructuredText {
            blocks: vec![TextBlock {
                block_id: "b".to_string(),
                text: "generated".to_string(),
                provenance: TextBlockProvenance::Generated,
                evidence_refs: vec![],
            }],
        },
        risk_tags: vec![],
        branch_label: None,
    };
    let err = ensure_generated_blocks_have_evidence(&[step]).expect_err("must fail");
    assert_eq!(err.code, AppErrorCode::ExportGateFailed);
}

#[test]
fn phase4_evidence_ids_stable_across_replay() {
    let _env_guard = env_lock();
    std::env::remove_var("OPSCINEMA_VISION_RAW_JSON");
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "evidence-replay".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::ocr::ocr_schedule(
        &backend,
        opscinema_types::OcrScheduleRequest {
            session_id: session.session_id,
            start_ms: Some(0),
            end_ms: Some(0),
        },
    )
    .expect("ocr");

    let set1 = api::evidence::evidence_for_time_range(
        &backend,
        opscinema_types::EvidenceForTimeRangeRequest {
            session_id: session.session_id,
            start_ms: 0,
            end_ms: i64::MAX,
        },
    )
    .expect("evidence 1");

    let conn2 = backend.storage.conn().expect("conn2");
    let set2 = crate::evidence::graph::derive_from_event_log(&conn2, session.session_id)
        .expect("evidence 2");

    let ids1 = set1
        .evidence
        .into_iter()
        .map(|e| e.evidence_id)
        .collect::<std::collections::BTreeSet<_>>();
    let ids2 = set2
        .evidence
        .into_iter()
        .map(|e| e.evidence_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids1, ids2);
    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
}

#[test]
fn phase5_replay_determinism_and_conflict_guard() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "steps-replay".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("gen");
    let listed = api::steps::steps_list(
        &backend,
        opscinema_types::StepsListRequest {
            session_id: session.session_id,
        },
    )
    .expect("list");
    let first = listed.steps.first().expect("step");

    let conflict = api::steps::steps_apply_edit(
        &backend,
        opscinema_types::StepsApplyEditRequest {
            session_id: session.session_id,
            base_seq: listed.head_seq + 1,
            op: opscinema_types::StepEditOp::UpdateTitle {
                step_id: first.step_id,
                title: "Changed".to_string(),
            },
        },
    )
    .expect_err("must conflict");
    assert_eq!(conflict.code, AppErrorCode::Conflict);

    let _ = api::steps::steps_apply_edit(
        &backend,
        opscinema_types::StepsApplyEditRequest {
            session_id: session.session_id,
            base_seq: listed.head_seq,
            op: opscinema_types::StepEditOp::UpdateTitle {
                step_id: first.step_id,
                title: "Changed".to_string(),
            },
        },
    )
    .expect("edit");

    let conn = backend.storage.conn().expect("conn");
    let replayed =
        crate::steps::replay::replay_session_steps(&conn, session.session_id).expect("replay");
    let listed2 = api::steps::steps_list(
        &backend,
        opscinema_types::StepsListRequest {
            session_id: session.session_id,
        },
    )
    .expect("list2");
    assert_eq!(replayed, listed2.steps);
}

#[test]
fn phase7_tutorial_strict_gate_blocks_warnings() {
    let input = ExportGateInput {
        steps: vec![],
        missing_evidence: vec![],
        degraded_anchor_ids: vec![],
        warnings: vec![opscinema_types::ExportWarning {
            code: "WARN".to_string(),
            message: "warn".to_string(),
        }],
    };
    let err = tutorial_pack_gate(&input).expect_err("must fail");
    assert_eq!(err.code, AppErrorCode::ExportGateFailed);
}

#[test]
fn phase6_anchor_degrade_blocks_tutorial_until_manual_fix() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "anchor-gate".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps");
    let step = api::steps::steps_list(
        &backend,
        opscinema_types::StepsListRequest {
            session_id: session.session_id,
        },
    )
    .expect("list")
    .steps
    .into_iter()
    .next()
    .expect("step");
    let anchors = api::anchors::anchors_list_for_step(
        &backend,
        opscinema_types::AnchorsListForStepRequest {
            session_id: session.session_id,
            step_id: step.step_id,
        },
    )
    .expect("anchors");
    let anchor = anchors.anchors[0].clone();

    let _ = api::anchors::anchors_reacquire(
        &backend,
        opscinema_types::AnchorsReacquireRequest {
            session_id: session.session_id,
            step_id: step.step_id,
        },
    )
    .expect("reacquire");

    let blocked = api::slicer::tutorial_validate_export(
        &backend,
        opscinema_types::TutorialValidateExportRequest {
            session_id: session.session_id,
        },
    )
    .expect("validate blocked");
    assert!(!blocked.allowed);

    let _ = api::anchors::anchors_manual_set(
        &backend,
        opscinema_types::AnchorsManualSetRequest {
            session_id: session.session_id,
            anchor_id: anchor.anchor_id,
            locators: vec![opscinema_types::EvidenceLocator {
                locator_type: opscinema_types::EvidenceLocatorType::AnchorBbox,
                asset_id: None,
                frame_ms: None,
                bbox_norm: Some(opscinema_types::BBoxNorm {
                    x: 1000,
                    y: 1000,
                    w: 2000,
                    h: 1000,
                }),
                text_offset: None,
                note: Some("manual anchor".to_string()),
            }],
            note: Some("manual fix".to_string()),
        },
    )
    .expect("manual set");

    let allowed = api::slicer::tutorial_validate_export(
        &backend,
        opscinema_types::TutorialValidateExportRequest {
            session_id: session.session_id,
        },
    )
    .expect("validate allowed");
    assert!(allowed.allowed);
}

#[test]
fn phase11_fixture_pipeline_export_verify_and_hash_regression() {
    let _env_guard = env_lock();
    std::env::remove_var("OPSCINEMA_VISION_RAW_JSON");
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "1");
    std::env::set_var("OPSCINEMA_DETERMINISTIC_IDS", "1");

    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);

    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "fixture".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");

    let _ = api::capture::capture_start(
        &backend,
        CaptureStartRequest {
            session_id: session.session_id,
        },
    )
    .expect("capture");

    let _ = api::ocr::ocr_schedule(
        &backend,
        opscinema_types::OcrScheduleRequest {
            session_id: session.session_id,
            start_ms: Some(0),
            end_ms: Some(0),
        },
    )
    .expect("ocr");

    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps");

    let steps = api::steps::steps_list(
        &backend,
        opscinema_types::StepsListRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps list")
    .steps;

    let out_dir = std::env::temp_dir().join(format!("opscinema-export-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");

    let export = tutorial_pack::export_tutorial_pack(
        session.session_id,
        &steps,
        tutorial_pack::TutorialPackBuildOptions {
            missing_evidence: vec![],
            degraded_anchor_ids: vec![],
            warnings: vec![],
            model_pins: vec![],
            offline_policy_enforced: true,
        },
        &out_dir,
    )
    .expect("export");
    let verify_res = verify::verify_bundle(Path::new(&export.output_path)).expect("verify");
    assert!(verify_res.valid, "issues: {:?}", verify_res.issues);

    let expected_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/fixtures/golden_session/expected_bundle_hash.txt");
    assert_or_update_hash(&expected_file, &export.bundle_hash, "golden_session");
    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
    std::env::remove_var("OPSCINEMA_DETERMINISTIC_IDS");
}

#[test]
fn phase11_tutorial_player_html_is_human_readable() {
    let step = Step {
        step_id: Uuid::new_v4(),
        order_index: 0,
        title: "Open billing dashboard".to_string(),
        body: StructuredText {
            blocks: vec![TextBlock {
                block_id: "b1".to_string(),
                text: "Open Billing and confirm invoice banner is visible".to_string(),
                provenance: TextBlockProvenance::Generated,
                evidence_refs: vec![Uuid::new_v4()],
            }],
        },
        risk_tags: vec![],
        branch_label: None,
    };
    let output_dir = std::env::temp_dir().join(format!("opscinema-player-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&output_dir).expect("mkdir");
    let _ = tutorial_pack::export_tutorial_pack(
        Uuid::new_v4(),
        &[step],
        tutorial_pack::TutorialPackBuildOptions {
            missing_evidence: vec![],
            degraded_anchor_ids: vec![],
            warnings: vec![],
            model_pins: vec![],
            offline_policy_enforced: true,
        },
        &output_dir,
    )
    .expect("export");
    let html = std::fs::read_to_string(output_dir.join("player/index.html")).expect("player");
    assert!(html.contains("OpsCinema Tutorial Pack"));
    assert!(html.contains("Evidence:"));
    assert!(html.contains("Tutorial strict: PASS"));
}

#[test]
#[ignore = "long-running soak; run explicitly in optional CI/manual workflows"]
fn phase11_capture_soak_stream_consistency() {
    let _env_guard = env_lock();
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "1");
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    std::env::set_var("OPSCINEMA_CAPTURE_BURST_FRAMES", "0");

    let soak_secs = std::env::var("OPSCINEMA_SOAK_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(8);
    let interval_ms = 180u32;

    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "capture-soak".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::capture::capture_set_config(opscinema_types::CaptureConfig {
        keyframe_interval_ms: interval_ms,
        include_input: false,
        include_window_meta: false,
    })
    .expect("set config");

    let started = api::capture::capture_start(
        &backend,
        CaptureStartRequest {
            session_id: session.session_id,
        },
    )
    .expect("start");
    assert_eq!(started.state, opscinema_types::CaptureState::Capturing);

    std::thread::sleep(std::time::Duration::from_secs(soak_secs));

    let keyframes = api::timeline::timeline_get_keyframes(
        &backend,
        opscinema_types::TimelineKeyframesRequest {
            session_id: session.session_id,
            start_ms: 0,
            end_ms: i64::MAX,
        },
    )
    .expect("timeline");
    // Capture cadence includes both configured sleep interval and frame processing overhead.
    // Budget a modest processing window so the soak assertion remains strict but non-flaky.
    let per_frame_budget_ms = u64::from(interval_ms) + 40;
    let expected_min = ((soak_secs * 1000) / per_frame_budget_ms).saturating_sub(1) as usize;
    assert!(
        keyframes.keyframes.len() >= expected_min,
        "expected at least {expected_min} frames with {per_frame_budget_ms}ms/frame budget, got {}",
        keyframes.keyframes.len()
    );

    let stopped = api::capture::capture_stop(
        &backend,
        opscinema_types::CaptureStopRequest {
            session_id: session.session_id,
        },
    )
    .expect("stop");
    assert_eq!(stopped.state, opscinema_types::CaptureState::Stopped);

    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
    std::env::remove_var("OPSCINEMA_CAPTURE_BURST_FRAMES");
}

#[test]
fn phase11_fixture_scenario_matrix_hash_regression() {
    let _env_guard = env_lock();
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "1");
    std::env::set_var("OPSCINEMA_DETERMINISTIC_IDS", "1");

    #[derive(Debug, Deserialize)]
    struct ScenarioAnchor {
        state: String,
    }
    #[derive(Debug, Deserialize)]
    struct ScenarioWarning {
        code: String,
        message: String,
    }
    #[derive(Debug, Deserialize)]
    struct ScenarioFixture {
        name: String,
        session: String,
        anchors: Option<Vec<ScenarioAnchor>>,
        warnings: Option<Vec<ScenarioWarning>>,
        expected_failures: Option<Vec<String>>,
    }

    let scenario_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/fixtures/scenarios");
    let mut scenario_files = std::fs::read_dir(&scenario_root)
        .expect("scenario dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    scenario_files.sort();
    assert!(
        !scenario_files.is_empty(),
        "expected at least one scenario fixture in {}",
        scenario_root.display()
    );
    for scenario_file in scenario_files {
        let scenario_json = std::fs::read_to_string(&scenario_file)
            .unwrap_or_else(|_| panic!("missing scenario {}", scenario_file.display()));
        let scenario: ScenarioFixture =
            serde_json::from_str(&scenario_json).expect("scenario parse");
        let scenario_name = scenario.name.clone();
        assert_eq!(scenario.name, scenario_name);

        if scenario
            .expected_failures
            .as_ref()
            .map(|f| f.iter().any(|v| v == "missing_evidence"))
            .unwrap_or(false)
        {
            let storage = Storage::open_in_memory().expect("storage");
            let backend = api::Backend::new(storage);
            let session = api::sessions::session_create(
                &backend,
                SessionCreateRequest {
                    label: scenario.session.clone(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("session");
            let _ = api::steps::steps_generate_candidates(
                &backend,
                opscinema_types::StepsGenerateCandidatesRequest {
                    session_id: session.session_id,
                },
            )
            .expect("steps");
            let listed = api::steps::steps_list(
                &backend,
                opscinema_types::StepsListRequest {
                    session_id: session.session_id,
                },
            )
            .expect("list");
            let first = listed.steps.first().expect("step");
            let _ = api::steps::steps_apply_edit(
                &backend,
                opscinema_types::StepsApplyEditRequest {
                    session_id: session.session_id,
                    base_seq: listed.head_seq,
                    op: opscinema_types::StepEditOp::ReplaceBody {
                        step_id: first.step_id,
                        body: StructuredText {
                            blocks: vec![TextBlock {
                                block_id: "strict-fail".to_string(),
                                text: "generated without evidence".to_string(),
                                provenance: TextBlockProvenance::Generated,
                                evidence_refs: vec![],
                            }],
                        },
                    },
                },
            )
            .expect("apply");
            let validate = api::slicer::tutorial_validate_export(
                &backend,
                opscinema_types::TutorialValidateExportRequest {
                    session_id: session.session_id,
                },
            )
            .expect("validate");
            assert!(!validate.allowed, "tutorial strict fixture must block");
            continue;
        }

        if scenario.warnings.as_ref().is_some_and(|w| !w.is_empty()) {
            let storage = Storage::open_in_memory().expect("storage");
            let backend = api::Backend::new(storage);
            let session = api::sessions::session_create(
                &backend,
                SessionCreateRequest {
                    label: scenario.session.clone(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("session");
            let _ = api::steps::steps_generate_candidates(
                &backend,
                opscinema_types::StepsGenerateCandidatesRequest {
                    session_id: session.session_id,
                },
            )
            .expect("steps");
            let warning = scenario
                .warnings
                .as_ref()
                .and_then(|v| v.first())
                .expect("warning");
            let mut conn = backend.storage.conn().expect("conn");
            let _ = event_store::append_event(
                &mut conn,
                session.session_id,
                "VerifierRunCompleted",
                &serde_json::json!({
                    "verifier_id":"shell.safe_echo",
                    "status":"FAILED",
                    "message": warning.message,
                    "code": warning.code
                }),
                None,
            )
            .expect("warning event");
            let out_dir = std::env::temp_dir().join(format!("opscinema-scenario-{scenario_name}"));
            std::fs::create_dir_all(&out_dir).expect("mkdir");
            let export = api::proof::proof_export_bundle(
                &backend,
                opscinema_types::ProofExportRequest {
                    session_id: session.session_id,
                    output_dir: out_dir.display().to_string(),
                },
            )
            .expect("proof export");
            let expected_file =
                scenario_root.join(format!("{scenario_name}.expected_bundle_hash.txt"));
            assert_or_update_hash(&expected_file, &export.bundle_hash, &scenario_name);
            continue;
        }

        if scenario.anchors.as_ref().is_some_and(|a| !a.is_empty()) {
            let storage = Storage::open_in_memory().expect("storage");
            let backend = api::Backend::new(storage);
            let session = api::sessions::session_create(
                &backend,
                SessionCreateRequest {
                    label: scenario.session.clone(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("session");
            let _ = api::capture::capture_start(
                &backend,
                CaptureStartRequest {
                    session_id: session.session_id,
                },
            )
            .expect("capture");
            let _ = api::ocr::ocr_schedule(
                &backend,
                opscinema_types::OcrScheduleRequest {
                    session_id: session.session_id,
                    start_ms: Some(0),
                    end_ms: Some(0),
                },
            )
            .expect("ocr");
            let _ = api::steps::steps_generate_candidates(
                &backend,
                opscinema_types::StepsGenerateCandidatesRequest {
                    session_id: session.session_id,
                },
            )
            .expect("steps");
            let steps = api::steps::steps_list(
                &backend,
                opscinema_types::StepsListRequest {
                    session_id: session.session_id,
                },
            )
            .expect("steps list")
            .steps;
            let first_step = steps.first().expect("first step");
            let anchors = api::anchors::anchors_list_for_step(
                &backend,
                opscinema_types::AnchorsListForStepRequest {
                    session_id: session.session_id,
                    step_id: first_step.step_id,
                },
            )
            .expect("anchors")
            .anchors;
            let first_anchor = anchors.first().expect("first anchor");
            if scenario
                .anchors
                .as_ref()
                .and_then(|v| v.first())
                .map(|a| a.state.contains("manual"))
                .unwrap_or(false)
            {
                let _ = api::anchors::anchors_manual_set(
                    &backend,
                    opscinema_types::AnchorsManualSetRequest {
                        session_id: session.session_id,
                        anchor_id: first_anchor.anchor_id,
                        locators: vec![opscinema_types::EvidenceLocator {
                            locator_type: opscinema_types::EvidenceLocatorType::AnchorBbox,
                            asset_id: None,
                            frame_ms: None,
                            bbox_norm: Some(opscinema_types::BBoxNorm {
                                x: 1000,
                                y: 1000,
                                w: 2000,
                                h: 1000,
                            }),
                            text_offset: None,
                            note: Some("manual fixture override".to_string()),
                        }],
                        note: Some("fixture-manual".to_string()),
                    },
                )
                .expect("manual set");
            } else {
                let _ = first_anchor;
            }

            let out_dir = std::env::temp_dir().join(format!("opscinema-scenario-{scenario_name}"));
            std::fs::create_dir_all(&out_dir).expect("mkdir");
            let export = api::exports::tutorial_export_pack(
                &backend,
                opscinema_types::TutorialExportRequest {
                    session_id: session.session_id,
                    output_dir: out_dir.display().to_string(),
                },
            )
            .expect("tutorial export");
            let expected_file =
                scenario_root.join(format!("{scenario_name}.expected_bundle_hash.txt"));
            assert_or_update_hash(&expected_file, &export.bundle_hash, &scenario_name);
            continue;
        }

        panic!("unknown scenario shape: {}", scenario_file.display());
    }

    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
    std::env::remove_var("OPSCINEMA_DETERMINISTIC_IDS");
}

#[test]
fn phase1_orphan_asset_gc_dry_run_and_delete() {
    let storage = Storage::open_in_memory().expect("storage");
    let mut conn = storage.conn().expect("conn");
    let store = asset_store::AssetStore::new(storage.assets_root.clone());

    let keep = store.put(&conn, b"keep", None).expect("keep");
    let orphan = store.put(&conn, b"orphan", None).expect("orphan");
    let session_id = Uuid::new_v4();

    conn.execute(
        "INSERT INTO sessions(session_id,label,created_at,head_seq,head_hash) VALUES (?1,'s',?2,0,'')",
        rusqlite::params![session_id.to_string(), crate::util::time::now_utc_iso()],
    )
    .expect("insert session");
    conn.execute(
        "INSERT INTO events(session_id,seq,event_id,event_type,payload_canon_json,prev_event_hash,event_hash,created_at)\n         VALUES (?1,1,?2,'Test','{\"asset_id\": \"x\"}',NULL,'h',?3)",
        rusqlite::params![session_id.to_string(), Uuid::new_v4().to_string(), crate::util::time::now_utc_iso()],
    )
    .expect("insert event");
    conn.execute(
        "UPDATE sessions SET head_seq=1, head_hash='h' WHERE session_id=?1",
        rusqlite::params![session_id.to_string()],
    )
    .expect("session head");
    conn.execute(
        "UPDATE events SET payload_canon_json=?1 WHERE rowid=(SELECT rowid FROM events LIMIT 1)",
        rusqlite::params![format!(r#"{{"asset_id":"{}"}}"#, keep)],
    )
    .expect("update payload");

    let dry = gc::gc_orphan_assets(&conn, &store, true).expect("dry gc");
    assert_eq!(dry.deleted_count, 0);
    assert!(dry.orphan_ids.contains(&orphan));

    let live = gc::gc_orphan_assets_with_audit(&mut conn, &store, false, Some(session_id))
        .expect("live gc");
    assert_eq!(live.deleted_count, 1);
    assert!(!store.path_for(&orphan).exists());
    assert!(store.path_for(&keep).exists());
    let events = event_store::query_events(&conn, session_id, None, 100).expect("events");
    assert!(events.iter().any(|e| e.event_type == "StorageGcRan"));
}

#[test]
fn phase8_verifier_result_is_persisted_and_fetchable() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "verifier".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::verifiers::verifier_run(
        &backend,
        opscinema_types::VerifierRunRequest {
            session_id: session.session_id,
            verifier_id: "shell.safe_echo".to_string(),
        },
    )
    .expect("verifier run");

    let conn = backend.storage.conn().expect("conn");
    let run_id_raw: String = conn
        .query_row(
            "SELECT run_id FROM verifier_runs ORDER BY created_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .expect("run id");
    let run_id = Uuid::parse_str(&run_id_raw).expect("parse run id");
    let result = api::verifiers::verifier_get_result(
        &backend,
        opscinema_types::VerifierGetResultRequest { run_id },
    )
    .expect("result");
    assert_eq!(result.verifier_id, "shell.safe_echo");
    assert!(!result.result_asset.asset_id.is_empty());
}

#[test]
fn phase8_runbook_is_replayed_and_export_is_listed() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "runbook".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps");
    let created = api::proof::runbook_create(
        &backend,
        opscinema_types::RunbookCreateRequest {
            session_id: session.session_id,
            title: "Initial".to_string(),
        },
    )
    .expect("create");
    let updated = api::proof::runbook_update(
        &backend,
        opscinema_types::RunbookUpdateRequest {
            runbook_id: created.runbook_id,
            title: Some("Updated".to_string()),
        },
    )
    .expect("update");
    assert_eq!(updated.title, "Updated");

    let out_dir = std::env::temp_dir().join(format!("opscinema-runbook-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let _ = api::proof::runbook_export(
        &backend,
        opscinema_types::RunbookExportRequest {
            runbook_id: created.runbook_id,
            output_dir: out_dir.display().to_string(),
        },
    )
    .expect("export");

    let listed = api::exports::exports_list(
        &backend,
        opscinema_types::ExportsListRequest {
            session_id: Some(session.session_id),
        },
    )
    .expect("list");
    assert!(
        !listed.exports.is_empty(),
        "runbook export should be persisted"
    );
}

#[test]
fn phase8_proof_and_runbook_exports_include_verifier_warnings() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "proof-warning".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps");
    let runbook = api::proof::runbook_create(
        &backend,
        opscinema_types::RunbookCreateRequest {
            session_id: session.session_id,
            title: "Proof Runbook".to_string(),
        },
    )
    .expect("runbook");

    let mut conn = backend.storage.conn().expect("conn");
    let _ = event_store::append_event(
        &mut conn,
        session.session_id,
        "VerifierRunCompleted",
        &serde_json::json!({
            "verifier_id":"shell.safe_echo",
            "status":"FAILED"
        }),
        None,
    )
    .expect("append warning event");

    let out_proof = std::env::temp_dir().join(format!("opscinema-proof-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_proof).expect("mkdir proof");
    let proof = api::proof::proof_export_bundle(
        &backend,
        opscinema_types::ProofExportRequest {
            session_id: session.session_id,
            output_dir: out_proof.display().to_string(),
        },
    )
    .expect("proof export");
    assert!(!proof.warnings.is_empty());

    let out_runbook = std::env::temp_dir().join(format!("opscinema-runbook-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_runbook).expect("mkdir runbook");
    let runbook_export = api::proof::runbook_export(
        &backend,
        opscinema_types::RunbookExportRequest {
            runbook_id: runbook.runbook_id,
            output_dir: out_runbook.display().to_string(),
        },
    )
    .expect("runbook export");
    assert!(!runbook_export.warnings.is_empty());
}

#[test]
fn phase8_proof_export_blocks_missing_generated_evidence() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "proof-coverage-block".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps");
    let listed = api::steps::steps_list(
        &backend,
        opscinema_types::StepsListRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps list");
    let first = listed.steps.first().expect("first");
    let _ = api::steps::steps_apply_edit(
        &backend,
        opscinema_types::StepsApplyEditRequest {
            session_id: session.session_id,
            base_seq: listed.head_seq,
            op: opscinema_types::StepEditOp::ReplaceBody {
                step_id: first.step_id,
                body: StructuredText {
                    blocks: vec![TextBlock {
                        block_id: "missing-proof".to_string(),
                        text: "generated body".to_string(),
                        provenance: TextBlockProvenance::Generated,
                        evidence_refs: vec![],
                    }],
                },
            },
        },
    )
    .expect("edit");

    let out_dir = std::env::temp_dir().join(format!("opscinema-proof-block-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let err = api::proof::proof_export_bundle(
        &backend,
        opscinema_types::ProofExportRequest {
            session_id: session.session_id,
            output_dir: out_dir.display().to_string(),
        },
    )
    .expect_err("must block");
    assert_eq!(err.code, AppErrorCode::ExportGateFailed);
}

#[test]
fn phase9_manifest_contains_model_pins_for_roles() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let model = api::model_dock::models_register(
        &backend,
        opscinema_types::ModelRegisterRequest {
            provider: "mlx".to_string(),
            label: "MLX QA".to_string(),
            model_name: "mlx-qa".to_string(),
            digest: "sha256:testdigest".to_string(),
        },
    )
    .expect("register model");
    let _ = api::model_dock::model_roles_set(
        &backend,
        opscinema_types::ModelRolesUpdate {
            tutorial_generation: Some(model.model_id.clone()),
            screen_explainer: None,
            anchor_grounding: None,
        },
    )
    .expect("set roles");

    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "manifest-pins".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps");

    let out_dir = std::env::temp_dir().join(format!("opscinema-pins-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let export = api::exports::tutorial_export_pack(
        &backend,
        opscinema_types::TutorialExportRequest {
            session_id: session.session_id,
            output_dir: out_dir.display().to_string(),
        },
    )
    .expect("tutorial export");
    let manifest_raw =
        std::fs::read_to_string(Path::new(&export.output_path).join("manifest.json"))
            .expect("manifest");
    let manifest: ExportManifestV1 = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(manifest.manifest_version, 1);
    assert_eq!(manifest.model_pins.len(), 1);
    assert_eq!(manifest.model_pins[0].role, "tutorial_generation");
    assert_eq!(manifest.model_pins[0].model_id, model.model_id);
}

#[test]
fn phase9_network_allowlist_blocks_and_allows_ollama() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);

    let blocked = api::model_dock::ollama_list(
        &backend,
        opscinema_types::OllamaListRequest {
            host: Some("blocked.local".to_string()),
        },
    )
    .expect_err("must block");
    assert_eq!(blocked.code, AppErrorCode::NetworkBlocked);

    let _ = api::app::network_allowlist_set(
        &backend,
        opscinema_types::NetworkAllowlistUpdate {
            entries: vec!["127.0.0.1".to_string()],
        },
    )
    .expect("allowlist set");
    let allowed = api::model_dock::ollama_list(
        &backend,
        opscinema_types::OllamaListRequest {
            host: Some("127.0.0.1".to_string()),
        },
    )
    .expect("allowed");
    assert!(!allowed.models.is_empty());

    let pull = api::model_dock::ollama_pull(
        &backend,
        opscinema_types::OllamaPullRequest {
            host: Some("127.0.0.1".to_string()),
            model: "llama3.1".to_string(),
        },
    )
    .expect("pull");
    let run = api::model_dock::ollama_run(
        &backend,
        opscinema_types::OllamaRunRequest {
            model_id: "ollama:digest".to_string(),
            prompt: "hello".to_string(),
        },
    )
    .expect("run");
    assert_ne!(pull.job_id, run.job_id);
}

#[test]
fn phase11_export_verify_fails_policy_attestation_mismatch() {
    let out_dir = std::env::temp_dir().join(format!("opscinema-verify-attest-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    std::fs::write(out_dir.join("tutorial.json"), b"{}").expect("write tutorial");

    let manifest = crate::exports::manifest::build_manifest(
        &out_dir,
        BundleType::TutorialPack,
        "session-test",
        vec![],
        PolicyAttestations {
            evidence_coverage_passed: false,
            tutorial_strict_passed: false,
            offline_policy_enforced: true,
        },
        vec![],
    )
    .expect("manifest");
    std::fs::write(
        out_dir.join("manifest.json"),
        to_canonical_json(&manifest).expect("canon"),
    )
    .expect("write manifest");

    let result = verify::verify_bundle(&out_dir).expect("verify");
    assert!(!result.valid);
    assert!(result
        .issues
        .iter()
        .any(|i| i.contains("policy attestation failed")));
}

#[test]
fn phase11_export_verify_allows_proof_warnings() {
    let out_dir = std::env::temp_dir().join(format!("opscinema-proof-warn-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    std::fs::write(out_dir.join("proof.json"), b"{}").expect("write proof");

    let manifest = crate::exports::manifest::build_manifest(
        &out_dir,
        BundleType::ProofBundle,
        "session-proof",
        vec![opscinema_export_manifest::ManifestWarning {
            code: "VERIFIER_WARN".to_string(),
            message: "warning allowed for proof bundle".to_string(),
        }],
        PolicyAttestations {
            evidence_coverage_passed: true,
            tutorial_strict_passed: false,
            offline_policy_enforced: true,
        },
        vec![],
    )
    .expect("manifest");
    std::fs::write(
        out_dir.join("manifest.json"),
        to_canonical_json(&manifest).expect("canon"),
    )
    .expect("write manifest");

    let result = verify::verify_bundle(&out_dir).expect("verify");
    assert!(result.valid, "proof warnings should be allowed");
    assert!(result.issues.is_empty());
}

#[test]
fn phase11_export_verify_rejects_tutorial_warnings() {
    let out_dir = std::env::temp_dir().join(format!("opscinema-tutorial-warn-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    std::fs::write(out_dir.join("tutorial.json"), b"{}").expect("write tutorial");

    let manifest = crate::exports::manifest::build_manifest(
        &out_dir,
        BundleType::TutorialPack,
        "session-tutorial",
        vec![opscinema_export_manifest::ManifestWarning {
            code: "WARN".to_string(),
            message: "tutorial warning".to_string(),
        }],
        PolicyAttestations {
            evidence_coverage_passed: true,
            tutorial_strict_passed: true,
            offline_policy_enforced: true,
        },
        vec![],
    )
    .expect("manifest");
    std::fs::write(
        out_dir.join("manifest.json"),
        to_canonical_json(&manifest).expect("canon"),
    )
    .expect("write manifest");

    let result = verify::verify_bundle(&out_dir).expect("verify");
    assert!(!result.valid);
    assert!(result
        .issues
        .iter()
        .any(|issue| issue.contains("tutorial bundle contains warnings")));
}

#[test]
fn phase11_export_verify_detects_bundle_hash_mismatch() {
    let out_dir = std::env::temp_dir().join(format!("opscinema-hash-mismatch-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    std::fs::write(out_dir.join("tutorial.json"), b"{}").expect("write tutorial");

    let mut manifest = crate::exports::manifest::build_manifest(
        &out_dir,
        BundleType::TutorialPack,
        "session-hash",
        vec![],
        PolicyAttestations {
            evidence_coverage_passed: true,
            tutorial_strict_passed: true,
            offline_policy_enforced: true,
        },
        vec![],
    )
    .expect("manifest");
    manifest.bundle_hash = "deadbeef".to_string();
    std::fs::write(
        out_dir.join("manifest.json"),
        to_canonical_json(&manifest).expect("canon"),
    )
    .expect("write manifest");

    let result = verify::verify_bundle(&out_dir).expect("verify");
    assert!(!result.valid);
    assert!(result
        .issues
        .iter()
        .any(|issue| issue.contains("bundle_hash mismatch")));
}

#[test]
fn phase10_agent_pipeline_is_explicit_and_event_sourced() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "agent".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");
    let _ = api::steps::steps_generate_candidates(
        &backend,
        opscinema_types::StepsGenerateCandidatesRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps");
    let listed = api::steps::steps_list(
        &backend,
        opscinema_types::StepsListRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps list");
    let first = listed.steps.first().expect("step");
    let _ = api::steps::steps_apply_edit(
        &backend,
        opscinema_types::StepsApplyEditRequest {
            session_id: session.session_id,
            base_seq: listed.head_seq,
            op: opscinema_types::StepEditOp::UpdateTitle {
                step_id: first.step_id,
                title: "Open  target  screen ".to_string(),
            },
        },
    )
    .expect("edit");

    let run = api::agent_plant::agent_pipeline_run(
        &backend,
        opscinema_types::AgentPipelineRunRequest {
            session_id: session.session_id,
            pipeline_id: "normalize_titles".to_string(),
        },
    )
    .expect("run");
    let report = api::agent_plant::agent_pipeline_report(
        &backend,
        opscinema_types::AgentPipelineReportRequest { run_id: run.job_id },
    )
    .expect("report");
    assert!(report.diagnostics.iter().any(|d| d.contains("run_id=")));

    let listed2 = api::steps::steps_list(
        &backend,
        opscinema_types::StepsListRequest {
            session_id: session.session_id,
        },
    )
    .expect("steps list after run");
    assert_eq!(listed2.steps[0].title, "Open target screen");
}

#[test]
fn phase11_jobs_list_filters_and_cancel_idempotent() {
    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "jobs-filter".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");

    let conn = backend.storage.conn().expect("conn");
    let queued_job =
        crate::storage::repo_jobs::create_job(&conn, "queued_job", Some(session.session_id))
            .expect("create queued");
    let other_job =
        crate::storage::repo_jobs::create_job(&conn, "other_job", None).expect("create other");
    crate::storage::repo_jobs::update_job_status(
        &conn,
        other_job,
        JobStatus::Succeeded,
        None,
        None,
    )
    .expect("mark other done");

    let filtered = api::jobs::jobs_list(
        &backend,
        opscinema_types::JobsListRequest {
            session_id: Some(session.session_id),
            status: Some(JobStatus::Queued),
        },
    )
    .expect("jobs list");
    assert_eq!(filtered.jobs.len(), 1);
    assert_eq!(filtered.jobs[0].job_id, queued_job);

    let cancel1 = api::jobs::jobs_cancel(
        &backend,
        opscinema_types::JobsCancelRequest { job_id: queued_job },
    )
    .expect("cancel1");
    let cancel2 = api::jobs::jobs_cancel(
        &backend,
        opscinema_types::JobsCancelRequest { job_id: queued_job },
    )
    .expect("cancel2");
    assert!(cancel1.accepted);
    assert!(cancel2.accepted);

    let cancelled = api::jobs::jobs_get(
        &backend,
        opscinema_types::JobsGetRequest { job_id: queued_job },
    )
    .expect("get cancelled");
    assert_eq!(cancelled.status, JobStatus::Cancelled);
    assert!(cancelled.ended_at.is_some());
}

#[test]
fn phase11_job_lifecycle_order_for_ocr_and_verifier() {
    let _env_guard = env_lock();
    std::env::set_var("OPSCINEMA_PROVIDER_MODE", "stub");
    std::env::set_var("OPSCINEMA_ASSUME_PERMISSIONS", "1");

    let storage = Storage::open_in_memory().expect("storage");
    let backend = api::Backend::new(storage);
    let session = api::sessions::session_create(
        &backend,
        SessionCreateRequest {
            label: "job-lifecycle".to_string(),
            metadata: BTreeMap::new(),
        },
    )
    .expect("session");

    let ocr_job = api::ocr::ocr_schedule(
        &backend,
        opscinema_types::OcrScheduleRequest {
            session_id: session.session_id,
            start_ms: Some(0),
            end_ms: Some(0),
        },
    )
    .expect("ocr");
    let verifier_job = api::verifiers::verifier_run(
        &backend,
        opscinema_types::VerifierRunRequest {
            session_id: session.session_id,
            verifier_id: "shell.safe_echo".to_string(),
        },
    )
    .expect("verifier");

    let ocr_detail = api::jobs::jobs_get(
        &backend,
        opscinema_types::JobsGetRequest {
            job_id: ocr_job.job_id,
        },
    )
    .expect("ocr job");
    let verifier_detail = api::jobs::jobs_get(
        &backend,
        opscinema_types::JobsGetRequest {
            job_id: verifier_job.job_id,
        },
    )
    .expect("verifier job");

    assert_eq!(ocr_detail.status, JobStatus::Succeeded);
    assert!(ocr_detail.started_at.is_some());
    assert!(ocr_detail.ended_at.is_some());
    assert_eq!(verifier_detail.status, JobStatus::Succeeded);
    assert!(verifier_detail.started_at.is_some());
    assert!(verifier_detail.ended_at.is_some());

    std::env::remove_var("OPSCINEMA_PROVIDER_MODE");
    std::env::remove_var("OPSCINEMA_ASSUME_PERMISSIONS");
}

#[test]
fn compile_time_boundary_forbids() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut tauri_violations = vec![];
    let mut db_violations = vec![];
    let mut fs_violations = vec![];

    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry.expect("walk");
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .expect("strip")
            .to_string_lossy()
            .to_string();
        let content = std::fs::read_to_string(path).expect("read");

        if content.contains("tauri::command") && !rel.starts_with("api/") {
            tauri_violations.push(rel.clone());
        }
        if (content.contains("rusqlite") || content.contains("sqlx"))
            && !rel.starts_with("storage/")
        {
            db_violations.push(rel.clone());
        }
        if (content.contains("std::fs::write") || content.contains("std::fs::File::create"))
            && !rel.starts_with("storage/")
            && !rel.starts_with("exports/")
            && !rel.starts_with("tests")
        {
            fs_violations.push(rel.clone());
        }
    }

    assert!(
        tauri_violations.is_empty(),
        "tauri violations: {tauri_violations:?}"
    );
    assert!(db_violations.is_empty(), "db violations: {db_violations:?}");
    assert!(fs_violations.is_empty(), "fs violations: {fs_violations:?}");
}
