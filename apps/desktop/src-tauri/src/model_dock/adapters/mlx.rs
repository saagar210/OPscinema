pub fn run_local(_model_id: &str, prompt: &str) -> anyhow::Result<String> {
    Ok(format!("mlx:{prompt}"))
}
