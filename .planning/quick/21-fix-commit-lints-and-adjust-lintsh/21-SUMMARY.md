# Summary: Fix commit lints and adjust lint.sh

## What was done
- Rewrote the messages for commits `docs(quick-20)` and `fix(ui)` to satisfy conventional commit formatting rules (max 100 chars per line).
- Updated `scripts/lint.sh` to properly lint commit messages even if the local working branch is directly `main`. It now checks the upstream branch difference, or falls back to `main..HEAD`, or just checks the very last commit (`HEAD~1..HEAD`). This ensures an agent making a local commit will get lint errors immediately when running `scripts/lint.sh`.

## Verification
- Local lint checks pass successfully.
