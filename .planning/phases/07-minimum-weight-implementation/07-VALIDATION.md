---
phase: 7
slug: minimum-weight-implementation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-03
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test / playwright / cucumber |
| **Config file** | none — Wave 0 installs |
| **Quick run command** | `cargo test` |
| **Full suite command** | `./scripts/ci-test.sh` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `./scripts/ci-test.sh`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 7-01-01 | 01 | 1 | CONF-01 | unit | `cargo test` | ✅ W0 | ⬜ pending |
| 7-01-02 | 01 | 1 | CONF-02 | unit | `cargo test` | ✅ W0 | ⬜ pending |
| 7-01-03 | 01 | 1 | CONF-03 | e2e | `./scripts/ci-test.sh` | ✅ W0 | ⬜ pending |
| 7-02-01 | 02 | 2 | SUGG-01 | unit | `cargo test` | ✅ W0 | ⬜ pending |
| 7-02-02 | 02 | 2 | SUGG-02 | e2e | `./scripts/ci-test.sh` | ✅ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/steps/workout_steps.rs` — update stubs for SUGG-01
- [ ] `tests/steps/library_steps.rs` — update stubs for CONF-03

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| File saving validation | CONF-01 | DB interactions in WASM | Verify database commits correctly via web UI |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
