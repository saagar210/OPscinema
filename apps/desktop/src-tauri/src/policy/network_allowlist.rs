use opscinema_types::{AppError, AppErrorCode, NetworkAllowlist, NetworkAllowlistUpdate};

#[derive(Debug, Clone, Default)]
pub struct NetworkPolicy {
    allowlist: Vec<String>,
}

impl NetworkPolicy {
    pub fn get(&self) -> NetworkAllowlist {
        NetworkAllowlist {
            entries: self.allowlist.clone(),
        }
    }

    pub fn set(&mut self, update: NetworkAllowlistUpdate) -> NetworkAllowlist {
        self.allowlist = update.entries;
        self.get()
    }

    pub fn check_host(&self, host: &str) -> Result<(), AppError> {
        if self.allowlist.iter().any(|e| e == host) {
            Ok(())
        } else {
            Err(AppError {
                code: AppErrorCode::NetworkBlocked,
                message: format!("Host {host} is not allowlisted"),
                details: None,
                recoverable: true,
                action_hint: Some("Open settings and add host to allowlist".to_string()),
            })
        }
    }
}
