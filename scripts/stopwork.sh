#!/bin/bash
# Wrapper so `stopwork` invokes Lantern stopwork.
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

exec "$LANTERN_BIN" stopwork "$@"
