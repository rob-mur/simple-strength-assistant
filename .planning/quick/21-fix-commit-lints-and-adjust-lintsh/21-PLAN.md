---
must_haves:
  - Fix the commit lint errors for the last two commits
  - Update scripts/lint.sh to catch bad commits for local agents more reliably
---

# Plan: Fix commit lints and adjust lint.sh

## Task 1: Fix Commit Messages

- **files:** `.git/COMMIT_EDITMSG` (via `git rebase` or `git commit --amend`)
- **action:** Reword the last two commits so their headers are under 100 chars and bodies wrap correctly under 100 chars.
- **verify:** `npx commitlint --from HEAD~2 --to HEAD --verbose`
- **done:** true

## Task 2: Adjust lint.sh

- **files:** `scripts/lint.sh`
- **action:** Update the local commitlint check to resolve `upstream` reference. If `main..HEAD` is empty (meaning the agent is on `main`), fallback to `HEAD~1..HEAD` to at least validate the most recent commit.
- **verify:** `scripts/lint.sh` runs successfully.
- **done:** true
