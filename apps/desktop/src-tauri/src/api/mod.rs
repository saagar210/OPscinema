pub mod agent_plant;
pub mod anchors;
pub mod app;
pub mod capture;
pub mod evidence;
pub mod exports;
pub mod jobs;
pub mod model_dock;
pub mod ocr;
pub mod proof;
#[cfg(feature = "runtime")]
pub mod runtime_events;
pub mod sessions;
pub mod slicer;
pub mod steps;
#[cfg(feature = "runtime")]
pub mod tauri_commands;
pub mod timeline;
pub mod verifiers;

use crate::jobs::runner::JobRunner;
use crate::policy::network_allowlist::NetworkPolicy;
use crate::storage::{asset_store::AssetStore, db::Storage};
use opscinema_types::{AppSettings, CaptureStatus};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct CaptureLoopControl {
    pub session_id: Uuid,
    pub stop: Arc<AtomicBool>,
}

pub type CaptureStatusHook = Arc<dyn Fn(CaptureStatus) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct Backend {
    pub storage: Arc<Storage>,
    pub assets: AssetStore,
    pub settings: Arc<Mutex<AppSettings>>,
    pub network_policy: Arc<Mutex<NetworkPolicy>>,
    pub capture_status: Arc<Mutex<CaptureStatus>>,
    pub capture_loop: Arc<Mutex<Option<CaptureLoopControl>>>,
    pub capture_status_hook: Arc<Mutex<Option<CaptureStatusHook>>>,
    pub jobs: JobRunner,
}

impl Backend {
    pub fn new(storage: Storage) -> Self {
        let assets = AssetStore::new(storage.assets_root.clone());
        Self {
            storage: Arc::new(storage),
            assets,
            settings: Arc::new(Mutex::new(AppSettings {
                offline_mode: true,
                allow_input_capture: false,
                allow_window_metadata: false,
            })),
            network_policy: Arc::new(Mutex::new(NetworkPolicy::default())),
            capture_status: Arc::new(Mutex::new(CaptureStatus {
                state: opscinema_types::CaptureState::Idle,
                session_id: None,
                started_at: None,
            })),
            capture_loop: Arc::new(Mutex::new(None)),
            capture_status_hook: Arc::new(Mutex::new(None)),
            jobs: JobRunner::default(),
        }
    }

    pub fn set_capture_status_hook(&self, hook: Option<CaptureStatusHook>) {
        if let Ok(mut slot) = self.capture_status_hook.lock() {
            *slot = hook;
        }
    }
}
