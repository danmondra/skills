#!/usr/bin/env bash
# Symlink every skill (outside deprecated/) into local harness dirs.
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
for target in "$HOME/.claude/skills" "$HOME/.agents/skills"; do
  mkdir -p "$target"
  for skill in "$REPO"/skills/*/*/; do
    [ -f "$skill/SKILL.md" ] || continue
    name="$(basename "$skill")"
    ln -sfn "$skill" "$target/$name"
  done
done
echo "Linked skills."
