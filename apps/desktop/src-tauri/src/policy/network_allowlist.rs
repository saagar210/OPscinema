use opscinema_types::{AppError, AppErrorCode, NetworkAllowlist, NetworkAllowlistUpdate};
use std::collections::BTreeSet;

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
        let normalized = update
            .entries
            .into_iter()
            .map(|entry| canonicalize_host(&entry))
            .filter(|entry| !entry.is_empty())
            .collect::<BTreeSet<_>>();
        self.allowlist = normalized.into_iter().collect();
        self.get()
    }

    pub fn check_host(&self, host: &str) -> Result<(), AppError> {
        let candidate = canonicalize_host(host);
        if self.allowlist.iter().any(|e| e == &candidate) {
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

fn canonicalize_host(raw: &str) -> String {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return String::new();
    }

    let no_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed.as_str());
    no_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::NetworkPolicy;
    use opscinema_types::{AppErrorCode, NetworkAllowlistUpdate};

    #[test]
    fn normalizes_entries_and_deduplicates() {
        let mut policy = NetworkPolicy::default();
        let updated = policy.set(NetworkAllowlistUpdate {
            entries: vec![
                " HTTPS://LOCALHOST:11434/api ".to_string(),
                "localhost:11434".to_string(),
                "".to_string(),
            ],
        });

        assert_eq!(updated.entries, vec!["localhost:11434".to_string()]);
        policy
            .check_host("https://localhost:11434/v1/models")
            .expect("normalized host should match");
    }

    #[test]
    fn rejects_unlisted_host_with_network_blocked_code() {
        let mut policy = NetworkPolicy::default();
        let _ = policy.set(NetworkAllowlistUpdate {
            entries: vec!["127.0.0.1".to_string()],
        });

        let err = policy.check_host("example.com").expect_err("must block");
        assert_eq!(err.code, AppErrorCode::NetworkBlocked);
        assert!(err.action_hint.is_some());
    }
}
