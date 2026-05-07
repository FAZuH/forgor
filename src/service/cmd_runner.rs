pub fn run_hook_command(cmd: &str) {
    let Some(parts) = shlex::split(cmd) else {
        log::error!("failed to parse hook command: {}", cmd);
        return;
    };
    let mut parts = parts.into_iter();
    let Some(prog) = parts.next() else { return };

    std::thread::spawn(move || {
        let output = std::process::Command::new(prog)
            .args(parts)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output();

        if let Ok(output) = output
            && !output.status.success()
        {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("hook command failed: {stderr}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_empty_string() {
        // Should not panic on empty command
        run_hook_command("");
    }

    #[test]
    fn cmd_whitespace() {
        run_hook_command("   ");
    }

    #[test]
    fn cmd_malformed_shlex() {
        // Unmatched quote should be handled gracefully (via shlex split returning None)
        run_hook_command("echo \"unmatched");
    }
}
