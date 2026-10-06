#!/usr/bin/env bash
# PostToolUse (Write|Edit): rustfmt the edited .rs file and flag the session so the
# Stop hook (clippy-gate.sh) runs clippy before Claude finishes.
input=$(cat)
root="${CLAUDE_PROJECT_DIR:-$(pwd)}"
file=$(jq -r '.tool_input.file_path // .tool_response.filePath // empty' <<<"$input")
case "$file" in "$root"/*.rs) ;; *) exit 0 ;; esac

session=$(jq -r '.session_id // "default"' <<<"$input")
touch "${TMPDIR:-/tmp}/claude-rs-edited-$session"

edition=$(sed -nE 's/^edition *= *"([0-9]+)".*/\1/p' "$root/Cargo.toml")
# Syntax errors are left to the Stop clippy gate, which reports them to Claude.
rustfmt --edition "${edition:-2021}" "$file" 2>/dev/null || true
