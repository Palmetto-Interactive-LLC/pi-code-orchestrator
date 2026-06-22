#!/bin/bash
# Wrapper so `startwork` invokes Lantern startwork.
set -euo pipefail

LANTERN_BIN="${HOME}/.lantern/bin/lantern"
if [[ ! -x "$LANTERN_BIN" ]]; then
  echo "error: $LANTERN_BIN not found. Run: lantern install" >&2
  exit 1
fi

if ! "$LANTERN_BIN" --version >/dev/null 2>&1; then
  echo "error: $LANTERN_BIN cannot run (macOS may have blocked it)." >&2
  echo "  Fix: codesign -s - -f \"$LANTERN_BIN\"   OR   lantern install" >&2
  exit 1
fi

if [[ $# -eq 1 && "$1" =~ ^(agi|codex|kimi|claude|ui|dat|ops|plt|sec|doc|qa)$ ]]; then
  exec "$LANTERN_BIN" startwork --agent "$1"
fi

exec "$LANTERN_BIN" startwork "$@"
