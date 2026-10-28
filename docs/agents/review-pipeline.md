# Review pipeline

Four review layers, each owning a different question so none repeats or overwrites another. All output posted on GitHub is in English.

| Layer | Runs on | Owns | Writes to |
|---|---|---|---|
| `/ponytail:ponytail-review` | Locally, before every PR into `release/*`, and on each `Address review` fix | Logic, safety, load, missing tests, speed, needless code. Every finding is fixed before the PR opens. | Nothing on GitHub; deliberately skipped findings are listed in the PR body |
| CI (`ci.yml`, `pr-title.yml`) | Every PR and push to `main` or `release/*`; `ci.yml` skips docs-only changes | `cargo fmt --check`, `clippy -D warnings`, `cargo test`, PR title format | Check runs |
| code-review (`code-review.yml`, runs `/mattpocock-skills:code-review` unchanged) | Release PR into `main`: once when it is opened non-draft or marked ready; again by hand after fixes | Standards (repo docs) and Spec (`Spec: #N` and `Closes #N` issues) | One comment starting with `<!-- mai-code-review -->`, updated in place |
| CodeRabbit (`.coderabbit.yaml`) | Release PR into `main`, non-draft, incremental on each push | Runtime defects: `unsafe`/FFI, Win32 handle leaks, error handling, security | Its own walkthrough and review comments; never the PR body |

## Rules that keep the layers apart

- Nothing but the author edits the PR body. CodeRabbit's summary stays in its walkthrough.
- Only the session watching the release PR pushes fixes. CodeRabbit's autofix and fix-CI are off, and the code-review workflow has read-only contents.
- The code-review workflow only edits its own marked comment, posted by `github-actions[bot]`.
- CodeRabbit does not reply automatically, so it never answers the code-review comment.

## Re-running code-review after fixes

```bash
gh workflow run code-review.yml -f pr=<release PR number>
```

It needs the `CLAUDE_CODE_OAUTH_TOKEN` repository secret (create it with `claude setup-token`).
