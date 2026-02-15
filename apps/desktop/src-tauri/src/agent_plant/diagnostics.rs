pub fn summarize(run_id: &str, steps: &[String]) -> Vec<String> {
    vec![
        format!("run_id={run_id}"),
        format!("transform_count={}", steps.len()),
    ]
}
