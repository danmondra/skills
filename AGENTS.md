# AGENTS.md

Skills live in bucket folders under `skills/` (e.g. `skills/personal/`).

- Each skill is a folder with a `SKILL.md` (required) with `name:` + `description:` frontmatter.
- Every skill promoted in `.claude-plugin/plugin.json`'s `skills` array must have an entry in the top-level `README.md` linking to its `SKILL.md`.
- After touching a manifest, run `claude plugin validate . --strict`.
- To link skills locally, run `scripts/link-skills.sh`.
