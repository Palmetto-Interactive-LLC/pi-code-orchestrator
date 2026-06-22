#!/usr/bin/env python3
"""
iterm_batch_init.py — Parallel Kimi init + title refresh.

Fire-and-retry: no startup grace, no 120s wait-for-ready gate.
Each pane gets its own fast retry loop (0.5s interval).
"""

import argparse
import asyncio
import json
import sys
from pathlib import Path

import iterm2

sys.path.insert(0, str(Path(__file__).resolve().parent))
from iterm_kimi_ready import inject_with_fast_retry  # noqa: E402


def _session_id_matches(candidate: str, target: str) -> bool:
    if candidate == target:
        return True
    if ":" in candidate and candidate.split(":", 1)[1] == target:
        return True
    if ":" in target and target.split(":", 1)[1] == candidate:
        return True
    return False


async def find_session(app: iterm2.App, session_id: str) -> iterm2.Session | None:
    session = app.get_session_by_id(session_id)
    if session is not None:
        return session
    for window in app.windows:
        for tab in window.tabs:
            for s in tab.sessions:
                if _session_id_matches(s.session_id, session_id):
                    return s
    return None


async def inject_agent_prompt(session: iterm2.Session, text: str) -> None:
    text = text.rstrip("\n")
    if not text:
        return
    await session.async_send_text(text)
    await asyncio.sleep(0.05)
    await session.async_send_text("\r")


async def apply_title(session: iterm2.Session, title: str) -> None:
    await session.async_set_name(title)
    osc = f"\x1b]0;{title}\x07\x1b]1;{title}\x07\x1b]2;{title}\x07"
    await session.async_inject(osc.encode("utf-8"))


async def inject_one(
    connection: iterm2.Connection,
    session_id: str,
    init_text: str,
) -> None:
    async def do_inject(session: iterm2.Session) -> None:
        await inject_agent_prompt(session, init_text)

    ok = await inject_with_fast_retry(
        connection,
        find_session,
        session_id,
        do_inject,
    )
    if not ok:
        print(f"WARN: init not confirmed in {session_id!r}", file=sys.stderr)


async def refresh_titles(
    connection: iterm2.Connection,
    sessions_by_role: dict[str, str],
    titles_by_role: dict[str, str],
) -> None:
    app = await iterm2.async_get_app(connection)
    tasks = []
    for role, iterm_id in sessions_by_role.items():
        title = titles_by_role.get(role)
        if not title:
            continue

        async def apply_one(sid: str = iterm_id, t: str = title) -> None:
            session = await find_session(app, sid)
            if session is not None:
                await apply_title(session, t)

        tasks.append(apply_one())
    if tasks:
        await asyncio.gather(*tasks)


async def main_async(
    connection: iterm2.Connection,
    init_by_role: dict[str, str],
    sessions_by_role: dict[str, str],
    titles_by_role: dict[str, str],
) -> None:
    init_tasks = []
    for role, init_text in init_by_role.items():
        sid = sessions_by_role.get(role)
        if sid and init_text:
            init_tasks.append(inject_one(connection, sid, init_text))
    if init_tasks:
        await asyncio.gather(*init_tasks)

    if titles_by_role:
        await refresh_titles(connection, sessions_by_role, titles_by_role)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--init-file", required=True)
    parser.add_argument("--sessions-file", required=True)
    parser.add_argument("--titles-file", default="")
    args = parser.parse_args()

    init_by_role = json.loads(Path(args.init_file).read_text(encoding="utf-8"))
    sessions_by_role = json.loads(Path(args.sessions_file).read_text(encoding="utf-8"))
    titles_by_role: dict[str, str] = {}
    if args.titles_file:
        path = Path(args.titles_file)
        if path.is_file():
            titles_by_role = json.loads(path.read_text(encoding="utf-8"))

    iterm2.run_until_complete(
        lambda conn: main_async(conn, init_by_role, sessions_by_role, titles_by_role)
    )
