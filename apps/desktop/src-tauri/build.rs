use chrono::{SecondsFormat, TimeZone, Utc};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=GIT_COMMIT");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo:rerun-if-env-changed=BUILD_TIMESTAMP_UTC");

    write_build_info();

    if std::env::var_os("CARGO_FEATURE_RUNTIME").is_some() {
        tauri_build::build();
    }
}

fn write_build_info() {
    let commit = std::env::var("GIT_COMMIT")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(git_commit_short)
        .unwrap_or_else(|| "dev".to_string());
    let built_at = build_timestamp_utc();

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let path = std::path::Path::new(&out_dir).join("build_info.rs");
    let content = format!(
        "pub const BUILD_GIT_COMMIT: &str = {:?};\n\
         pub const BUILD_TIMESTAMP_UTC: &str = {:?};\n",
        commit, built_at
    );
    std::fs::write(path, content).expect("write build_info.rs");
}

fn git_commit_short() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn build_timestamp_utc() -> String {
    if let Ok(value) = std::env::var("BUILD_TIMESTAMP_UTC") {
        if !value.trim().is_empty() {
            return value;
        }
    }

    if let Ok(value) = std::env::var("SOURCE_DATE_EPOCH") {
        if let Ok(epoch) = value.parse::<i64>() {
            if let Some(dt) = Utc.timestamp_opt(epoch, 0).single() {
                return dt.to_rfc3339_opts(SecondsFormat::Secs, true);
            }
        }
    }

    "1970-01-01T00:00:00Z".to_string()
}
