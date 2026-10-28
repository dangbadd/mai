## Agent skills

### Issue tracker

Issues live in GitHub Issues for `dangbadd/mai` (via the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

### Branches

One `release/v<x.y.z>` branch per Release; one ticket branch and PR into it per ticket. See `docs/agents/branch-naming.md`.

### PR titles

`<type>(v<x.y.z>): <Description> (#<issue>)` into `release/*`, `release(v<x.y.z>): <Spec title> (#<spec>)` into `main`. See `docs/agents/pr-naming.md`.

### Before opening a PR into `release/*`

1. Run `/ponytail:ponytail-review` on `git diff origin/release/v<x.y.z>...HEAD`.
2. Fix every finding; list any you deliberately skip, with the reason, in the PR body.
3. Re-run `cargo test`.
4. Title the PR per `docs/agents/pr-naming.md` and write the body with the `pr` skill.
