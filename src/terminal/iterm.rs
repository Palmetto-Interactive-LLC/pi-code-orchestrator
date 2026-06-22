//! iTerm2 control via shipped Python helpers (iterm2 package).

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::process::Command;

/// Locate a Python helper script shipped alongside the Lantern binary.
pub fn locate_script(name: &str) -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let installed = home.join(".lantern").join("bin").join(name);
        if installed.exists() {
            return Ok(installed);
        }
    }

    let dev_path = std::env::current_dir()
        .unwrap_or_default()
        .join("src")
        .join("startwork")
        .join(name);
    if dev_path.exists() {
        return Ok(dev_path);
    }

    anyhow::bail!(
        "Cannot locate helper script '{}'. Run lantern install or use cargo run from the crate root.",
        name
    )
}

/// Capture visible text from an iTerm2 session by ID.
pub async fn capture_text(iterm_session_id: &str) -> Result<String> {
    let script = r#"
import asyncio
import sys
import iterm2

async def main(connection):
    app = await iterm2.async_get_app(connection)
    for window in app.windows:
        for tab in window.tabs:
            for session in tab.sessions:
                if session.session_id == sys.argv[1]:
                    screen = await session.async_get_screen_contents()
                    print("\n".join(screen.line(row).string for row in range(screen.number_of_lines)))
                    return
    raise SystemExit(2)

iterm2.run_until_complete(main)
"#;

    let pythons = [
        std::env::var("LANTERN_PYTHON").ok(),
        Some("/opt/homebrew/bin/python3.14".to_string()),
        Some("python3".to_string()),
    ];
    let mut last_err = String::new();
    for py in pythons.into_iter().flatten() {
        match Command::new(&py)
            .args(["-c", script, iterm_session_id])
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
            Ok(output) => {
                last_err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            }
            Err(e) => last_err = format!("{py}: {e}"),
        }
    }

    anyhow::bail!("failed to capture iTerm2 session {iterm_session_id}: {last_err}")
}

/// Close the iTerm2 window that contains panes for this devorch session.
pub async fn close_window(session_id: &str) -> Result<()> {
    let script_path = locate_script("iterm_close.py")?;

    let output = Command::new("python3")
        .args([
            script_path.to_str().context("non-UTF-8 script path")?,
            "--session",
            session_id,
        ])
        .output()
        .await
        .context("failed to launch iterm_close.py")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(session_id, "iterm_close.py: {}", stderr.trim());
    }

    Ok(())
}
