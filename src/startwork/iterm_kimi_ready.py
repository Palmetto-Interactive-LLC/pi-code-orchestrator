"""Session discovery + MCP-ready gate for Kimi pane injection."""

from __future__ import annotations

import asyncio
import re

import iterm2

DEFAULT_RETRY_INTERVAL_S = 0.3
DEFAULT_MAX_ATTEMPTS = 20   # ~6s to find the iTerm session
MCP_WAIT_INTERVAL_S = 0.5
MCP_WAIT_MAX_S = 35.0       # give up waiting for MCP after 35s and inject anyway

# Kimi shows "MCP Servers: 0/1 connected" or "connecting to mcp server" while loading.
# We look for either:
#   a) the loading indicator has disappeared (no "MCP Servers:.*0/" text) AND
#      the kimi interactive prompt (❯) is visible.
#   b) timeout — inject anyway.
_MCP_STILL_LOADING_RE = re.compile(
    r"MCP Servers:.*\b0/\d+|connecting to mcp server",
    re.IGNORECASE,
)
# Kimi's interactive prompt character — only shows after kimi fully starts.
# Shell prompts use $ or %, never ❯.
_KIMI_PROMPT_RE = re.compile(r"❯\s*$", re.MULTILINE)


async def _get_pane_text(session: iterm2.Session) -> str:
    """Return visible screen text from the session (best-effort)."""
    try:
        screen = await session.async_get_screen_contents()
        lines = [screen.line(row).string for row in range(screen.number_of_lines)]
        return "\n".join(lines)
    except Exception:
        return ""


async def _wait_for_kimi_mcp_ready(
    connection: iterm2.Connection,
    find_session_fn,
    session_id: str,
) -> iterm2.Session | None:
    """
    Poll until kimi shows its interactive prompt (❯) with no MCP loading indicator.
    Falls through after MCP_WAIT_MAX_S so the init prompt still gets injected.
    """
    deadline = asyncio.get_event_loop().time() + MCP_WAIT_MAX_S
    session = None

    while asyncio.get_event_loop().time() < deadline:
        app = await iterm2.async_get_app(connection)
        session = await find_session_fn(app, session_id)
        if session is None:
            await asyncio.sleep(MCP_WAIT_INTERVAL_S)
            continue

        text = await _get_pane_text(session)

        # Kimi prompt is visible AND no MCP loading indicator → ready
        if _KIMI_PROMPT_RE.search(text) and not _MCP_STILL_LOADING_RE.search(text):
            return session

        await asyncio.sleep(MCP_WAIT_INTERVAL_S)

    # Timed out — return whatever session we have
    if session is None:
        app = await iterm2.async_get_app(connection)
        session = await find_session_fn(app, session_id)
    return session


async def inject_with_fast_retry(
    connection: iterm2.Connection,
    find_session_fn,
    session_id: str,
    inject_fn,
    *,
    retry_interval_s: float = DEFAULT_RETRY_INTERVAL_S,
    max_attempts: int = DEFAULT_MAX_ATTEMPTS,
) -> bool:
    """
    Wait for kimi's interactive prompt to be ready (MCP connected),
    then inject the init prompt.
    """
    session = await _wait_for_kimi_mcp_ready(connection, find_session_fn, session_id)
    if session is None:
        return False

    try:
        await inject_fn(session)
        return True
    except Exception:
        return False
