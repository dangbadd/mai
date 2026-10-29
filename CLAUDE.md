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

### Running `/mattpocock-skills:implement`

Invoking it on a ticket is standing approval for every step up to the merge. Do not stop to ask between steps:

1. Create the ticket branch from `release/v<x.y.z>` (see `docs/agents/branch-naming.md`).
2. Implement and commit. Use `/ponytail:ponytail-review` in place of the skill's `code-review` step (see `docs/agents/review-pipeline.md`).
3. Run the "Before opening a PR into `release/*`" checklist.
4. Push the branch and open the PR.
5. Turn on Auto-fix for the PR and fix CI until it is green.
6. Only then ask one question: squash-merge the PR into `release/v<x.y.z>`?

Ask earlier only when blocked: the ticket is ambiguous, an action is destructive (force-push, deleting a branch), or a design decision is not settled by the ticket.

### Review pipeline

Ponytail locally, CI on every PR, CodeRabbit and code-review only on release PRs into `main`. See `docs/agents/review-pipeline.md`.
