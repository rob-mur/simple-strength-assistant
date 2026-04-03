---
name: ralph
description: Orchestrate the full ralph issue loop — assess each issue then implement it. Use when invoked as /ralph [scope], or when the user wants to run the automated backlog processing loop.
---

# Ralph

Orchestrate the ralph issue loop: for each issue in scope, assess it with ralph-assess, then implement it with ralph-code, collecting the resulting PRs.

## Invocation

```
/ralph [scope]
```

`scope` is an optional natural language argument describing which issues to process (e.g. "issue 42", "the auth issues", "everything tagged backend"). If omitted, the default scope is all open backlog issues.

## Responsibilities

1. Resolve the scope argument into a concrete list of issue IDs
2. For each issue ID, run ralph-assess as a sub-agent
3. On `STATUS: blocked`: log the issue as skipped and move on
4. On `STATUS: ready`: run ralph-code as a sub-agent
5. On `STATUS: blocked` from ralph-code: log the issue as skipped
6. On `STATUS: ready` from ralph-code: record the PR that was raised
7. After the queue is exhausted, output the list of raised PRs

## Step 1 — Resolve Scope

Interpret the scope argument as a set of issues to process.

- If no argument was given: list all open issues in the backlog. Use whatever forge or backlog tool is available in the current environment (gh, glab, gitea CLI, forgejo CLI, curl against a REST API, etc.). Collect their IDs.
- If an argument was given: interpret it as best you can. Examples:
  - `42` → issue 42 only
  - `42 43 44` → those three issues
  - `the authentication issues` → list open issues and filter to those whose title or labels relate to authentication
  - `everything tagged backend` → list open issues with the `backend` label

If after interpreting the argument you are still uncertain which issues are in scope — for example because the label does not exist, the description matches many unrelated issues, or the argument is inherently ambiguous — **stop and ask the user for clarification before proceeding**. Do not guess.

Once scope is resolved, log the list of issue IDs you are about to process so the user can see the plan.

## Step 2 — Issue Loop

Process each issue in the resolved list **sequentially** (not in parallel).

For each issue ID:

### 2a — Assess

Launch ralph-assess as a sub-agent, passing only the issue ID:

```
/ralph-assess <issue-id>
```

The sub-agent fetches its own context. Do not pass any additional information.

Read the STATUS line — it is always the last line of the sub-agent's response.

- `STATUS: blocked <reason>`: log the issue as skipped with the reason. Do not proceed to implementation. Move on to the next issue.
- `STATUS: ready`: proceed to 2b.

### 2b — Implement

Launch ralph-code as a sub-agent, passing only the issue ID:

```
/ralph-code <issue-id>
```

The sub-agent fetches its own context. Do not pass any additional information.

Read the STATUS line — it is always the last line of the sub-agent's response.

- `STATUS: blocked <reason>`: log the issue as skipped with the reason. Move on to the next issue.
- `STATUS: ready`: extract the PR identifier from the sub-agent's response (the PR URL or number it reported when raising the PR) and add it to the PR list.

## Step 3 — CI Monitoring Loop

After the issue queue is exhausted, take the list of PRs collected in Step 2 and enter the CI monitoring loop. Skip this step entirely if no PRs were raised.

### 3a — Polling

Poll CI status for every PR that is not yet in a terminal state. A PR is terminal when it is:

- **green**: CI has passed (all required checks successful)
- **merged**: the PR has been merged into the target branch
- **blocked**: a fix sub-agent returned `STATUS: blocked` (see 3b)

Poll at approximately **2-minute intervals**. Use whatever forge API or CLI is available:

- GitHub: `gh pr checks <pr-id>` and `gh pr view <pr-id> --json state`
- GitLab: `glab mr checks <mr-id>` or REST API for pipeline status
- Forgejo / Gitea: REST API via `curl` for the PR and its associated CI run

Each polling round: check all non-terminal PRs in sequence. After checking all, wait ~2 minutes before the next round.

### 3b — CI Failure Handling

When a CI check on a PR is in a failed (not pending) state:

1. Launch `ralph-code` as a sub-agent in fix mode, passing only the PR identifier:

   ```
   /ralph-code --fix-ci <pr-id>
   ```

2. Read the STATUS line — it is always the last line of the sub-agent's response.
   - `STATUS: blocked <reason>`: mark the PR as **blocked** and record the reason. Remove it from the polling list.
   - `STATUS: ready`: the sub-agent has pushed a fix. Continue polling this PR normally (CI will re-run).

Do not launch a new fix sub-agent for a PR while a fix sub-agent for that same PR is still running. If CI fails again after a fix was pushed, launch another fix sub-agent following the same rules.

### 3c — Timeout

Each PR has a maximum polling window of **~30 minutes** from the time it first entered the polling list (i.e. from when it was raised in Step 2). If the 30-minute window expires and the PR is still not in a terminal state, mark it as **blocked** with reason `CI monitoring timeout`.

### 3d — Termination

Exit the CI monitoring loop when every tracked PR is in a terminal state (green, merged, or blocked). Then proceed to Step 4.

## Step 4 — Final Summary Report

Output a single consolidated report:

```
## Ralph Run Complete

### PRs Raised
- <pr-url-or-id> (issue <issue-id>: <issue-title>)
- ...

### PRs Blocked
- <pr-url-or-id>: <reason from fix sub-agent or "CI monitoring timeout">
- ...

### Skipped Issues
- Issue <issue-id>: <one-line reason>
- ...
```

Rules for the report:

- **PRs Raised**: list every PR that reached a terminal state of green or merged. Include the forge URL or identifier, the originating issue ID, and the issue title.
- **PRs Blocked**: list every PR whose CI could not be fixed. Include the reason supplied by the fix sub-agent or `CI monitoring timeout`. Omit this section if there are none.
- **Skipped Issues**: list every issue that was skipped (blocked at assess or implement stage) with the one-line reason. Omit this section if there are none.
- If no PRs were raised at all, say so explicitly and omit the PRs Raised section.

## Sub-Agent Model

- Each sub-agent receives only the issue ID or PR identifier — it is responsible for fetching all context it needs from the forge.
- The STATUS line is always the very last line of the sub-agent's response. Read it literally; do not infer status from other content.
- Sub-agents run sequentially. Do not launch a ralph-code sub-agent while a ralph-assess sub-agent is still running, and do not begin the next issue until the current one is fully resolved.
- During CI monitoring, do not launch a new fix sub-agent for a PR while an existing fix sub-agent for that PR is still running.

## Forge Agnosticism

Scope resolution and issue listing are expressed as intent. Use whatever CLI, API, or project skill is available. Common examples:

- GitHub: `gh issue list --state open`, `gh issue list --label backend --state open`
- GitLab: `glab issue list --state opened`, `glab issue list --label backend`
- Forgejo / Gitea: REST API via `curl` or a project-specific CLI

If you are unsure which forge is in use, inspect the git remote URL, look for a `.claude/` project skill, or check for installed CLIs.

## Design Constraints

- No configuration files or environment variables are read. All context comes from the issues themselves and Claude's knowledge of the project.
- This skill covers the full lifecycle: issue assessment, implementation, CI monitoring, and final reporting.
