# Claude Agent Guide

## Dev Environment

This project uses [devenv](https://devenv.sh). All tools (cargo, rustfmt, clippy, prettier, lint, build, dx, npm, playwright) live in the nix shell — they are **not** on the system PATH by default.

Activate before running any tooling:

```bash
direnv allow
eval "$(direnv export bash)"
```

Run this once per session. After that, `cargo`, `lint`, `build`, `prettier`, etc. are all available in the current shell.

The pre-commit hooks (lint, build, rustfmt, prettier) require this environment. Git commits and pushes that trigger hooks must be run after the above export, or they will fail with `No such file or directory`.

## Getting CI Logs

The Forgejo instance exposes Actions logs via `fj-ex` (not the standard REST API):

```bash
# List jobs for a run
fj-ex actions jobs -H forgejo.clarob.uk -r rob/simple-strength-assistant --run-index <RUN_ID>

# Get logs for a specific job
fj-ex actions logs job \
  -H forgejo.clarob.uk \
  -r rob/simple-strength-assistant \
  --run-index <RUN_ID> \
  --job-index <JOB_INDEX>

# Get logs for all jobs in a run
fj-ex actions logs run \
  -H forgejo.clarob.uk \
  -r rob/simple-strength-assistant \
  --run-index <RUN_ID>
```

The run ID and job index are visible in the CI status `target_url` field, e.g. `/actions/runs/302/jobs/0` → run 302, job index 0.

To find the run ID for a PR, check the commit statuses:

```bash
curl -s "https://forgejo.clarob.uk/api/v1/repos/rob/simple-strength-assistant/commits/<SHA>/statuses" \
  | python3 -c "import json,sys; [print(s['context'], s['status'], s['target_url']) for s in json.load(sys.stdin)]"
```

## Forgejo CLI

Authenticated as `rob`. Always pass `-H forgejo.clarob.uk` or the host can't be inferred:

```bash
fj -H forgejo.clarob.uk pr list
fj -H forgejo.clarob.uk pr edit rob/simple-strength-assistant#<N> title "<new title>"
```

Token is stored at `/env/.local/share/forgejo-cli/keys.json`. For git push with credentials:

```bash
git push https://rob:<token>@forgejo.clarob.uk/rob/simple-strength-assistant.git <branch>
```

## Lint

The `lint` script (`scripts/lint.sh`) checks in order:

1. Commit messages (commitlint, `@commitlint/config-conventional`)
2. PR title (same rules — subject must be lowercase-starting, e.g. `feat(scope): add something`)
3. `package-lock.json` sync
4. `public/styles.css` sync with `src/styles.css` (run `npm run build:css` if out of sync)
5. `cargo fmt -- --check`
6. `prettier --check .`
7. `cargo clippy -- -D warnings`

Common gotcha: **commit subject must start with a lowercase word**. `FAB + Show Archived toggle` fails; `add FAB and show-archived toggle` passes. Acronyms like FAB are fine mid-subject but not as the first word.

If `public/styles.css` is out of sync, run:

```bash
npm run build:css
git add public/styles.css
```

## Pushing

Force-push is sometimes needed after amending. The pre-push hook also runs lint+build (slow — ~2 min):

```bash
eval "$(direnv export bash)"
git push https://rob:<token>@forgejo.clarob.uk/rob/simple-strength-assistant.git <branch> --force
```
