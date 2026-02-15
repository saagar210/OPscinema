use anyhow::Context;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const MAX_TIMEOUT_SECS: u64 = 30;

pub fn run_shell(
    allowed: &[String],
    cmd: &str,
    args: &[&str],
    timeout_secs: u64,
) -> anyhow::Result<String> {
    if !allowed.iter().any(|c| c == cmd) {
        anyhow::bail!("command not allowed")
    }
    if timeout_secs > MAX_TIMEOUT_SECS {
        anyhow::bail!("timeout too high")
    }
    if is_destructive(cmd, args) {
        anyhow::bail!("destructive command is blocked")
    }

    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("run {cmd}"))?;

    let deadline = Instant::now() + Duration::from_secs(timeout_secs.max(1));
    loop {
        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            let mut text = String::new();
            text.push_str(&String::from_utf8_lossy(&output.stdout));
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                text.push_str("\n[stderr]\n");
                text.push_str(&stderr);
            }
            if !status.success() {
                anyhow::bail!("command failed: {}", text.trim());
            }
            return Ok(text);
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("command timed out");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn is_destructive(cmd: &str, args: &[&str]) -> bool {
    let blocked = [
        "rm", "mv", "dd", "diskutil", "chmod", "chown", "truncate", "mkfs", "sudo",
    ];
    if blocked.iter().any(|b| b == &cmd) {
        return true;
    }

    let args_lc = args
        .iter()
        .map(|a| a.to_ascii_lowercase())
        .collect::<Vec<_>>();
    args_lc
        .iter()
        .any(|a| is_destructive_arg(a) || has_destructive_flag(a))
}

fn is_destructive_arg(arg: &str) -> bool {
    arg.contains("--delete")
        || arg.contains("/dev/")
        || arg.contains("sudo")
        || arg == "-rf"
        || arg == "-fr"
        || arg == "-r"
        || arg == "-f"
}

fn has_destructive_flag(arg: &str) -> bool {
    arg.starts_with('-')
        && arg.len() > 2
        && arg.contains('r')
        && arg.contains('f')
}

#[cfg(test)]
mod tests {
    use super::run_shell;

    #[test]
    fn blocks_destructive_commands() {
        let err =
            run_shell(&["rm".to_string()], "rm", &["-rf", "/tmp/x"], 1).expect_err("must block");
        assert!(err.to_string().contains("destructive"));
    }

    #[test]
    fn blocks_excessive_timeout() {
        let err = run_shell(&["echo".to_string()], "echo", &["ok"], 300).expect_err("must block");
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn blocks_delete_style_flags_for_allowlisted_commands() {
        let err = run_shell(
            &["rsync".to_string()],
            "rsync",
            &["--delete", "src", "dst"],
            1,
        )
        .expect_err("must block");
        assert!(err.to_string().contains("destructive"));
    }

    #[test]
    fn blocks_compound_rf_flag_for_allowlisted_commands() {
        let err = run_shell(&["echo".to_string()], "echo", &["-rf"], 1).expect_err("must block");
        assert!(err.to_string().contains("destructive"));
    }
}
