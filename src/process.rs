#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result, anyhow};

#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<CmdOutput> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd
        .output()
        .with_context(|| format!("failed to execute {program}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let args_str = args.join(" ");
        return Err(anyhow!(
            "command failed: {program} {args_str}\nexit: {status}\nstderr:\n{stderr}",
            status = output.status
        ));
    }

    Ok(CmdOutput {
        status: output.status,
        stdout,
        stderr,
    })
}

pub fn run_stdout(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<String> {
    Ok(run(program, args, cwd)?.stdout)
}
