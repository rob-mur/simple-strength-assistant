---
status: testing
phase: 03-verify-and-polish
source: [.planning/phases/03-verify-and-polish/03-01-PLAN.md]
started: 2026-02-26T17:00:00Z
updated: 2026-02-26T17:00:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 7
name: PWA Install Prompt (Mobile)
expected: |
  1. Deploy to Vercel.
  2. Open on mobile device.
  3. Verify manifest.json loads without 401 error (using 'crossorigin="use-credentials"').
  4. Verify PWA install prompt/banner appears in the SelectingFile state.
awaiting: user response

## Tests

### 1. PWA Assets Presence in Build
expected: 'manifest.json' and 'service-worker.js' are present in 'dist/public/'.
result: pass
note: "Verified that 'dx bundle' correctly copies all assets from 'public/' to 'dist/public/'. Pinned dioxus to 0.7.2 to match CLI version."

### 2. File Handle Persistence (Chrome)
expected: Selecting a file, granting permission, and refreshing should auto-initialize to Ready state without re-prompting.
result: [pending]

### 3. Database Initialization Success (New File)
expected: Creating a new .sqlite file should initialize successfully and create tables.
result: [pending]

### 4. LocalStorage Fallback Activation (Firefox)
expected: In Firefox, app should auto-activate fallback storage and show "Browser Storage Mode" banner.
result: [pending]

### 5. Invalid File Format Error UI
expected: Selecting a .txt file should show "Invalid File Format" error with a "Select Different File" button.
result: [pending]

### 6. User Cancellation Error UI
expected: Cancelling the file picker should show "File Selection Cancelled" error with a "Select File" retry button.
result: [pending]

### 7. PWA Install Prompt (Mobile)
expected: On mobile devices, an install banner should appear in the SelectingFile state if not already installed.
result: [pending]

## Summary

total: 7
passed: 1
issues: 0
pending: 6
skipped: 0

## Gaps

[none yet]
