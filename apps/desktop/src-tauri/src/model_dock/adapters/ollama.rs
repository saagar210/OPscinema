use crate::policy::network_allowlist::NetworkPolicy;

pub fn list_models(policy: &NetworkPolicy, host: &str) -> anyhow::Result<Vec<String>> {
    policy.check_host(host)?;
    Ok(vec!["llama3.1".to_string(), "qwen2.5".to_string()])
}

pub fn pull_model(policy: &NetworkPolicy, host: &str, model: &str) -> anyhow::Result<String> {
    policy.check_host(host)?;
    Ok(format!("pulled:{host}:{model}"))
}

pub fn run_prompt(
    policy: &NetworkPolicy,
    host: &str,
    model_id: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    policy.check_host(host)?;
    Ok(format!("ollama:{host}:{model_id}:{prompt}"))
}
