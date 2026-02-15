use opscinema_types::{AppError, AppErrorCode, PermissionsStatus};

pub fn require_screen_permission(status: &PermissionsStatus) -> Result<(), AppError> {
    if status.screen_recording {
        Ok(())
    } else {
        Err(AppError {
            code: AppErrorCode::PermissionDenied,
            message: "Screen recording permission missing".to_string(),
            details: None,
            recoverable: true,
            action_hint: Some("Open macOS Settings > Privacy > Screen Recording".to_string()),
        })
    }
}
