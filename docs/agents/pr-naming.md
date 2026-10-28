# PR titles

The `pr-title` CI check enforces these.

## Ticket PR into `release/*`

`<type>(v<x.y.z>): <Description> (#<issue>)`

- **type** matches the branch prefix: `feat`, `fix`, `docs`, `chore`.
- **Description**: English, starts with a capital letter, no trailing period.
- Drop ` (#<issue>)` when the work has no issue.
- The PR is squash-merged, so this title becomes the commit message on the release branch.

Examples: `feat(v0.1.0): Mai stands idle on the Ground (#3)`, `chore(v0.1.0): Review pipeline`.

## Release PR into `main`

`release(v<x.y.z>): <Spec title> (#<spec issue>)`

Example: `release(v0.1.0): Mai v0.1.0 — desktop Character on the taskbar (#2)`.
