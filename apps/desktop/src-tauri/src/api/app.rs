use crate::api::Backend;
use crate::policy::permissions::require_screen_permission;
use crate::verifiers::builtins::macos_settings;
use opscinema_types::{
    AppError, AppResult, AppSettings, BuildInfo, NetworkAllowlist, NetworkAllowlistUpdate,
    PermissionsStatus,
};

pub fn app_get_build_info() -> AppResult<BuildInfo> {
    Ok(BuildInfo {
        app_name: "OpsCinema Suite".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        commit: option_env!("GIT_COMMIT").unwrap_or("dev").to_string(),
        built_at: chrono::Utc::now(),
    })
}

pub fn app_get_permissions_status() -> AppResult<PermissionsStatus> {
    if let Ok(value) = std::env::var("OPSCINEMA_ASSUME_PERMISSIONS") {
        if value == "1" {
            return Ok(PermissionsStatus {
                screen_recording: true,
                accessibility: true,
                full_disk_access: true,
            });
        }
        if value == "0" {
            return Ok(PermissionsStatus {
                screen_recording: false,
                accessibility: false,
                full_disk_access: false,
            });
        }
    }

    Ok(PermissionsStatus {
        screen_recording: macos_settings::screen_recording_enabled(),
        accessibility: macos_settings::accessibility_enabled(),
        full_disk_access: macos_settings::full_disk_access_enabled(),
    })
}

pub fn settings_get(backend: &Backend) -> AppResult<AppSettings> {
    backend
        .settings
        .lock()
        .map(|s| s.clone())
        .map_err(|_| internal("settings lock poisoned"))
}

pub fn settings_set(backend: &Backend, req: AppSettings) -> AppResult<AppSettings> {
    backend
        .settings
        .lock()
        .map(|mut s| {
            *s = req.clone();
            req
        })
        .map_err(|_| internal("settings lock poisoned"))
}

pub fn network_allowlist_get(backend: &Backend) -> AppResult<NetworkAllowlist> {
    backend
        .network_policy
        .lock()
        .map(|p| p.get())
        .map_err(|_| internal("network policy lock poisoned"))
}

pub fn network_allowlist_set(
    backend: &Backend,
    req: NetworkAllowlistUpdate,
) -> AppResult<NetworkAllowlist> {
    backend
        .network_policy
        .lock()
        .map(|mut p| p.set(req))
        .map_err(|_| internal("network policy lock poisoned"))
}

pub fn assert_capture_allowed() -> AppResult<()> {
    let status = app_get_permissions_status()?;
    require_screen_permission(&status)?;
    Ok(())
}

fn internal(message: &str) -> AppError {
    AppError {
        code: opscinema_types::AppErrorCode::Internal,
        message: message.to_string(),
        details: None,
        recoverable: false,
        action_hint: None,
    }
}
