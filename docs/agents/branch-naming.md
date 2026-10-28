# Branch workflow

Each Release is built on its own release branch. Tickets land on that branch through small PRs; only the release branch's PR into `main` gets the full review. See `review-pipeline.md` for which review runs where.

## Release branch

- Named `release/v<x.y.z>`, cut from the latest `main` when work on that version starts.
- Only one release branch is open at a time.
- Its PR into `main` is opened as a **draft** as soon as the branch has its first commit, titled per `pr-naming.md`. The body lists `Spec: #<spec issue>` and a `Closes #<n>` line for every ticket merged into the branch, for traceability (those tickets are already closed, see below).
- Mark the PR **Ready for review** once every ticket of the milestone is merged. That starts the full review.
- Review findings are fixed with `Address review: ...` commits pushed straight to the release branch, after `cargo test` and `/ponytail:ponytail-review` on the fix. Then re-run code-review once: `gh workflow run code-review.yml -f pr=<N>`. Do not use CodeRabbit's "Fix comments" autofix.
- Merged into `main` with a **merge commit**. Then tag the merge commit `v<x.y.z>` (push the tag only when the owner agrees). The branch is kept but never committed to again; the next Release, Patches included, is cut from `main`.

## Ticket branch

- Named `<type>/v<x.y.z>-<issue>-<slug>`, cut from the open release branch; its PR targets that release branch.
  - **type**: `feat` (new behaviour), `fix` (bug fix), `docs` (documentation only), `chore` (tooling, config, scripts).
  - **v<x.y.z>**: the release branch's version.
  - **issue**: the GitHub issue number. Omit it, with its dash, when the work has no issue.
  - **slug**: a few lowercase English words joined by `-`.
- Merged into the release branch with **squash**, so the PR title becomes that branch's commit message.
- Right after the merge, close the ticket with a comment `Merged into release/v<x.y.z> via #<PR>`. GitHub only auto-closes issues on merges into `main`, and an open ticket would keep blocking the tickets that depend on it.

Examples: `release/v0.1.0`, `feat/v0.1.0-3-idle-on-ground`, `fix/v0.1.1-12-tray-menu-stuck`, `docs/v0.1.0-branch-naming` (no issue).
