use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::Result;

use crate::error::WtError;

#[derive(Debug, Clone)]
pub struct CmdOutput {
    #[allow(dead_code)] // status field kept for completeness, may be used in future
    pub status: ExitStatus,
    pub stdout: String,
    #[allow(dead_code)] // stderr field kept for completeness, may be used in future
    pub stderr: String,
}

pub fn run(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<CmdOutput> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd.output().map_err(|e| {
        WtError::io_error_with_source(format!("failed to execute {}", program), e.into())
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let args_str = args.join(" ");
        return Err(WtError::git_error(format!(
            "command failed: {} {}\nexit: {}\nstderr:\n{}",
            program, args_str, output.status, stderr
        ))
        .into());
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
