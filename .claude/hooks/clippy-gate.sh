#!/usr/bin/env bash
# Stop: if Claude edited .rs files in this session (flag set by rustfmt-edited.sh),
# `cargo clippy -- -D warnings` must pass before it finishes.
input=$(cat)
session=$(jq -r '.session_id // "default"' <<<"$input")
flag="${TMPDIR:-/tmp}/claude-rs-edited-$session"
[ -f "$flag" ] || exit 0
command -v cargo >/dev/null || exit 0
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0

if out=$(cargo clippy --quiet --message-format short -- -D warnings 2>&1); then
  rm -f "$flag"
  exit 0
fi
out=$(printf '%s\n' "$out" | head -n 60)

if [ "$(jq -r '.stop_hook_active // false' <<<"$input")" = "true" ]; then
  # Claude already had one round to fix it: let it stop, but tell the user.
  rm -f "$flag"
  jq -n --arg m "cargo clippy -- -D warnings still fails:
$out" '{systemMessage: $m}'
  exit 0
fi

jq -n --arg r "cargo clippy -- -D warnings fails after your Rust edits. Fix it before finishing; if your changes did not cause it, tell the user instead.
$out" '{decision: "block", reason: $r}'
