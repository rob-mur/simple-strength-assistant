---
status: testing
phase: 03-verify-and-polish
source: [.planning/phases/03-verify-and-polish/03-01-PLAN.md]
started: 2026-02-26T17:00:00Z
updated: 2026-02-26T19:00:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 6
name: User Cancellation Error UI
expected: |
  1. Open the app in Chrome.
  2. Click "Open Existing Database".
  3. Close the file picker dialog without selecting a file (e.g., press Escape).
  4. App should show a "File Selection Cancelled" error card with a "Select File" retry button.
awaiting: user response

## Tests

### 1. PWA Assets Presence in Build
expected: 'manifest.json' and 'service-worker.js' are present in 'dist/public/'.
result: pass
note: "Verified that 'dx bundle' correctly copies all assets from 'public/' to 'dist/public/'. Pinned dioxus to 0.7.2 to match CLI version."

### 2. File Handle Persistence & Re-requesting (Chrome)
expected: |
  - Selecting a file, granting permission, and refreshing should auto-initialize to Ready state.
  - If permission is lost (e.g., browser restart), app should show "Grant Permission" button instead of the file picker.
  - Clicking "Grant Permission" should prompt for permission on the *same* file.
result: pass
note: "Verified that permission can be re-requested for cached handles without re-opening the file picker."

### 3. Database Initialization Success (New File)
expected: Creating a new .sqlite file should initialize successfully and create tables.
result: pass
note: "Verified that 'Create New Database' correctly creates a file, initializes schema, and enters Ready state."

### 4. LocalStorage Fallback Activation (Firefox)
expected: In Firefox, app should auto-activate fallback storage and show "Browser Storage Mode" banner.
result: pass
note: "Verified that fallback to LocalStorage auto-activates in Firefox with an info banner."

### 5. Invalid File Format Error UI
expected: Selecting a .txt file should show "Invalid File Format" error with a "Select Different File" button.
result: pass
note: "Verified that selecting a .txt file triggers the correct Error UI. Fixed a bug where invalid handles were being cached, causing a loop."

### 6. User Cancellation Error UI
expected: Cancelling the file picker should show "File Selection Cancelled" error with a "Select File" retry button.
result: [pending]

### 7. PWA Install Prompt (Mobile)
expected: On mobile devices, an install banner should appear in the SelectingFile state if not already installed.
result: pass
note: "Fixed manifest 401 error with crossorigin='use-credentials'. PWA is installable on mobile Chrome."

## Summary

total: 7
passed: 6
issues: 0
pending: 1
skipped: 0

## Gaps

[none yet]
