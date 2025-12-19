#![allow(dead_code)]

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};

/// Configuration options for fzf
#[derive(Debug, Clone)]
pub struct FzfOptions {
    pub height: String,
    pub layout: String,
    pub preview: Option<String>,
    pub preview_window: String,
    pub prompt: Option<String>,
    pub header: Option<String>,
}

impl Default for FzfOptions {
    fn default() -> Self {
        Self {
            height: "40%".to_string(),
            layout: "reverse".to_string(),
            preview: None,
            preview_window: "right:60%".to_string(),
            prompt: None,
            header: None,
        }
    }
}

/// Run fzf with the given candidates and options.
///
/// Returns the selected line, or None if user cancelled (Esc/Ctrl-C).
/// The candidates are newline-separated strings piped to fzf stdin.
///
/// # Arguments
/// * `candidates` - List of strings to display in fzf
/// * `options` - Configuration options for fzf behavior and appearance
///
/// # Returns
/// * `Ok(Some(line))` - User selected a line
/// * `Ok(None)` - User cancelled (Esc/Ctrl-C) or no match
/// * `Err(_)` - Error occurred (e.g., fzf not installed)
pub fn run_fzf(candidates: &[String], options: &FzfOptions) -> Result<Option<String>> {
    // Build fzf command arguments
    let mut args = vec![
        "--height".to_string(),
        options.height.clone(),
        "--layout".to_string(),
        options.layout.clone(),
        "--preview-window".to_string(),
        options.preview_window.clone(),
    ];

    if let Some(preview) = &options.preview {
        args.push("--preview".to_string());
        args.push(preview.clone());
    }

    if let Some(prompt) = &options.prompt {
        args.push("--prompt".to_string());
        args.push(prompt.clone());
    }

    if let Some(header) = &options.header {
        args.push("--header".to_string());
        args.push(header.clone());
    }

    // Spawn fzf process
    let mut child = Command::new("fzf")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn fzf (is it installed?)")?;

    // Write candidates to stdin
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("failed to open fzf stdin"))?;

        for candidate in candidates {
            writeln!(stdin, "{}", candidate).context("failed to write to fzf stdin")?;
        }
        // stdin is dropped here, closing the pipe
    }

    // Wait for fzf to complete and capture output
    let output = child
        .wait_with_output()
        .context("failed to wait for fzf to complete")?;

    // Handle exit codes
    match output.status.code() {
        Some(0) => {
            // User made a selection
            let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if selection.is_empty() {
                Ok(None)
            } else {
                Ok(Some(selection))
            }
        }
        Some(1) => {
            // No match found
            Ok(None)
        }
        Some(130) => {
            // User cancelled (Ctrl-C or Esc)
            Ok(None)
        }
        Some(code) => Err(anyhow!("fzf exited with unexpected code: {}", code)),
        None => Err(anyhow!("fzf was terminated by a signal")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fzf_options_default() {
        let opts = FzfOptions::default();
        assert_eq!(opts.height, "40%");
        assert_eq!(opts.layout, "reverse");
        assert_eq!(opts.preview_window, "right:60%");
        assert!(opts.preview.is_none());
        assert!(opts.prompt.is_none());
        assert!(opts.header.is_none());
    }

    #[test]
    fn test_fzf_options_custom() {
        let opts = FzfOptions {
            height: "50%".to_string(),
            layout: "default".to_string(),
            preview: Some("cat {}".to_string()),
            preview_window: "up:40%".to_string(),
            prompt: Some("Select> ".to_string()),
            header: Some("Pick one:".to_string()),
        };

        assert_eq!(opts.height, "50%");
        assert_eq!(opts.layout, "default");
        assert_eq!(opts.preview, Some("cat {}".to_string()));
        assert_eq!(opts.preview_window, "up:40%");
        assert_eq!(opts.prompt, Some("Select> ".to_string()));
        assert_eq!(opts.header, Some("Pick one:".to_string()));
    }
}
