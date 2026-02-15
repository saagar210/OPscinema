use crate::policy::network_allowlist::NetworkPolicy;

pub fn host_allowed(policy: &NetworkPolicy, host: &str) -> bool {
    policy.check_host(host).is_ok()
}
