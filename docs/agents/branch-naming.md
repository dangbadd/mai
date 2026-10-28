# Branch naming

Every change goes on its own branch and lands through a PR, one PR per ticket. Never commit to `main` directly.

## Format

`<type>/v<x.y.z>-<issue>-<slug>`

- **type**: `feat` (new behaviour), `fix` (bug fix), `docs` (documentation only), `chore` (tooling, config, scripts).
- **v<x.y.z>**: the version the work ships in, taken from the issue's milestone (e.g. `v0.1.0`). A Patch after a Release uses the next `z` (e.g. `v0.1.1`).
- **issue**: the GitHub issue number. Omit it, with its dash, when the work has no issue.
- **slug**: a few lowercase English words joined by `-`.

## Examples

- `feat/v0.1.0-3-idle-on-ground`
- `fix/v0.1.1-12-tray-menu-stuck`
- `docs/v0.1.0-branch-naming` (no issue)

The PR body says `Closes #<issue>` so merging closes the ticket.
