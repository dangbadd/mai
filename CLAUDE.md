## Agent skills

### Issue tracker

Issues live in GitHub Issues for `dangbadd/mai` (via the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

### Branches

One branch and PR per ticket, named `<type>/v<x.y.z>-<issue>-<slug>`. See `docs/agents/branch-naming.md`.

### Review pipeline

Ponytail locally, CI on every PR, CodeRabbit and code-review only on release PRs into `main`. See `docs/agents/review-pipeline.md`.
