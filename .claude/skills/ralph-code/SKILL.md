---
name: ralph-code
description: Implement a backlog issue using TDD (if available), then branch off main and raise a PR. Use when invoked as /ralph-code <issue-id>, or when the ralph orchestrator needs to implement a pre-assessed issue. Also supports CI-fix mode: /ralph-code --fix-ci <pr-id> fetches CI failure logs for an existing PR and pushes a fix to its branch.
---

# Ralph Code

Implement a single backlog issue that has already been assessed, then raise a pull request for review.

## Invocation

```
/ralph-code <issue-id>          # implement an issue and raise a PR (default mode)
/ralph-code --fix-ci <pr-id>    # fix CI failures on an existing PR (CI-fix mode)
```

## Responsibilities

1. Fetch the issue details and QA plan by the given ID
2. Implement the feature (using the TDD skill if installed)
3. On test failure: retry up to 5 times, feeding error output back into each subsequent attempt
4. On max retries exceeded: tag/comment the issue as blocked with the reason, then output `STATUS: blocked <one-line reason>`
5. On success: branch off main, raise a PR, then output `STATUS: ready`

## Step 1 — Fetch the Issue and QA Plan

Retrieve the issue by ID using whatever forge or backlog tool is available in the current environment (gh, glab, gitea CLI, forgejo CLI, curl against a REST API, etc.). You do not need to be told which one to use — figure it out from the project context.

Collect:

- Title
- Description / body
- All comments — specifically locate the QA checklist comment posted by ralph-assess

The QA checklist is your acceptance definition. Each item in the checklist is a behaviour the implementation must satisfy. If no QA checklist comment is found, proceed using the issue description as your acceptance definition, but note the absence in your working notes.

## Step 2 — Choose Implementation Approach

Before writing any code, check whether the TDD skill is installed:

- If `~/.claude/skills/tdd/` exists: invoke `/tdd` and follow its red-green-refactor workflow throughout the implementation.
- If it does not exist: implement without it, writing tests alongside the code where the project conventions support it.

In either case, derive the test cases from the QA checklist items — each behavioural check should correspond to at least one test.

## Step 3 — Implement

Implement the feature. Follow the project's existing conventions for language, structure, naming, and test runner.

Keep the implementation focused on satisfying the QA checklist. Do not introduce unrelated changes, refactors, or speculative features.

Run the tests after completing the implementation.

## Step 4 — Retry Loop

If tests fail, do not give up immediately. Retry up to **5 times** in total:

1. Capture the full test failure output.
2. Analyse the failure — identify the specific assertion or error and its likely cause.
3. Revise the implementation to address the failure.
4. Run the tests again.

Count each run after the first as a retry. If you reach 5 retries without passing tests, proceed to the blocked path (Step 5a).

If tests pass at any point during the retry loop, proceed to the ready path (Step 5b).

## Step 5a — Blocked Path

If the implementation could not be completed after 5 retries:

1. Post a comment on the issue explaining what was attempted and why it could not be completed. Include the last test failure output. Be specific so a developer knows what to fix.
2. If the forge supports labels/tags, add a `blocked` label (or equivalent) to the issue.
3. Output (as the very last line):

```
STATUS: blocked <one-line reason>
```

The reason should name the specific obstacle (e.g. `STATUS: blocked tests fail with dependency injection error after 5 retries`).

## Step 5b — Ready Path

Once all tests pass:

1. Determine the branch name from the issue title:
   - Convert the title to lowercase
   - Replace spaces and special characters with hyphens
   - Prefix with the issue ID (e.g. `42-add-user-export-endpoint`)
   - Keep it short — truncate at a natural word boundary around 50 characters if needed

2. Create the branch off the current default branch (typically `main`):
   - Use git to create and switch to the new branch
   - Stage and commit all changes with a clear commit message referencing the issue ID

3. Raise a pull request:
   - Use whatever forge tool is available to open a PR from the new branch targeting `main`
   - Title: match the issue title
   - Body: include a reference to the issue, a brief description of what was implemented, and the QA checklist from the issue comment (copy it across so reviewers can verify without leaving the PR)

4. Output (as the very last line):

```
STATUS: ready
```

## Forge Agnosticism

All forge operations — fetching issues, posting comments, adding labels, creating branches, raising PRs — are expressed as intent. Use whatever CLI, API, or project skill is available. Common examples:

- GitHub: `gh issue view <id>`, `gh issue comment <id> --body "..."`, `gh issue edit <id> --add-label blocked`, `gh pr create --base main --head <branch> --title "..." --body "..."`
- GitLab: `glab issue view <id>`, `glab issue note create <id> --message "..."`, `glab mr create --source-branch <branch> --target-branch main`
- Forgejo / Gitea: REST API via `curl` or a project-specific CLI
- YouTrack / Jira: REST API or project-specific skill

If you are unsure which forge is in use, inspect the git remote URL, look for a `.claude/` project skill, or check for installed CLIs.

## Output Protocol

The **very last line** of your response must be one of:

```
STATUS: ready
STATUS: blocked <one-line reason>
```

No trailing text, blank lines, or punctuation after the STATUS line.

---

# CI-Fix Mode

Invoked as `/ralph-code --fix-ci <pr-id>`.

CI-fix mode operates entirely within the existing PR branch. It does **not** create a new branch or open a new PR.

## CI-Fix Step 1 — Fetch PR Details and CI Failure Logs

Retrieve the PR by ID using whatever forge tool is available.

Collect:

- PR title and description
- Target branch and head branch name
- Current CI/check status — only failed or errored checks are relevant
- The full log output for each failed check/job

Use whatever mechanism the forge exposes:

- GitHub: `gh pr view <pr-id>`, `gh pr checks <pr-id>`, `gh run view <run-id> --log-failed`
- GitLab: `glab mr view <pr-id>`, `glab ci view`, pipeline job traces via REST API
- Forgejo / Gitea: REST API (`/api/v1/repos/{owner}/{repo}/pulls/{pr-id}`, `/api/v1/repos/{owner}/{repo}/statuses/{sha}`)
- Other forges: use `curl` or a project-specific CLI to retrieve job logs

If CI is currently in progress (not yet finished), wait briefly and re-poll before proceeding. Do not attempt a fix against a still-running pipeline.

## CI-Fix Step 2 — Classify the Failure

Read the failure logs carefully and classify the root cause into one of two categories:

**Fixable** — the failure is caused by something in this repository that can be changed:

- Test assertions failing due to a code defect
- Compilation or type errors introduced by the PR
- Linting or formatting violations
- Missing or broken fixtures, snapshots, or generated files
- Configuration errors in CI workflow files within the repository

**Unfixable** — the failure is caused by something outside the repository's control:

- External dependency outage (package registry down, third-party API unavailable)
- Resource quota exceeded on the CI provider
- Flaky external service (network timeout connecting to a service not owned by this project)
- CI infrastructure failure (runner crash unrelated to code)
- A required secret or environment variable is absent from the CI configuration and cannot be added by a code change alone

If the classification is ambiguous, lean toward attempting a fix. Only declare unfixable when the log evidence clearly points to an external cause.

## CI-Fix Step 3a — Unfixable Path

If the failure is unfixable:

1. Post a comment on the PR explaining:
   - Which check(s) failed
   - The specific log evidence that indicates an external cause
   - Why a code change cannot resolve it
   - Any suggested manual action (e.g. "retry the pipeline once the registry is back", "add the missing secret `FOO_API_KEY` in the repo CI settings")

2. Output (as the very last line):

```
STATUS: blocked <one-line reason>
```

Example: `STATUS: blocked npm registry timeout — external outage, not a code defect`

## CI-Fix Step 4 — Attempt the Fix

Check out the PR's head branch locally, then apply the fix.

```
git fetch origin
git checkout <head-branch>
```

Follow the same implementation discipline as the standard implement mode:

- Make only the changes needed to address the CI failure
- Do not introduce unrelated edits, refactors, or feature work
- Run the failing check(s) locally if possible to verify the fix before pushing

Apply the retry loop from the standard mode (up to **5 attempts**) if local verification is available. If the CI environment cannot be reproduced locally (e.g. a platform-specific runner, unavailable secrets), make a single best-effort fix based on the log analysis and proceed directly to push.

## CI-Fix Step 5a — Fix Retry Limit Exceeded

If local verification is available and 5 retries are exhausted without a passing result:

1. Post a comment on the PR describing what was attempted and the last failure output.
2. Output (as the very last line):

```
STATUS: blocked <one-line reason>
```

Example: `STATUS: blocked local tests still fail after 5 retries — see PR comment`

## CI-Fix Step 5b — Push the Fix

Once the fix is ready:

1. Stage and commit the changes with a clear message referencing the PR and the failure:
   - Format: `fix(ci): resolve <short description of failure> (#<pr-id>)`

2. Push to the existing head branch — **do not force-push** unless the branch history makes a normal push impossible and you are certain it is safe:

```
git push origin <head-branch>
```

3. Do not open a new PR. The fix commit will appear on the existing PR automatically.

4. Output (as the very last line):

```
STATUS: ready
```
