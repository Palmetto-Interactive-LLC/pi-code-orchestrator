//! iTerm display helpers for Lantern launcher lifecycle.

use crate::types::TerminalTarget;

pub mod iterm;
pub use iterm::locate_script;

/// Close the squad window for a devorch session ID (iTerm2 only).
pub async fn close_session_window(session_id: &str) -> anyhow::Result<()> {
    iterm::close_window(session_id).await
}

/// Whether this target uses iTerm2.
pub fn is_iterm(target: &TerminalTarget) -> bool {
    target.transport_status == "ready" && !target.iterm_session_id.is_empty()
}
