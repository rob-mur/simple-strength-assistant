# External Integrations

**Analysis Date:** 2026-02-25

## APIs & External Services

**No External APIs:**
- Application is fully client-side with no external service integrations
- All data processing occurs in-browser via WebAssembly

## Data Storage

**Databases:**
- SQLite (sql.js WASM build)
  - Connection: In-memory or loaded from File System Access API
  - Client: Custom JavaScript bridge in `public/db-module.js`
  - Tables: `sessions`, `completed_sets`, `exercises`
  - Location: Browser via File System Access API or fallback storage

**File Storage:**
- File System Access API (primary)
  - File handle persistence via IndexedDB
  - Storage module: `public/file-handle-storage.js`
  - Rust wrapper: `src/state/file_system.rs`
  - File type: `.sqlite`, `.db` files
  - User-selected location on local filesystem
- LocalStorage (fallback)
  - Key: `workout_db_data`
  - Used when File System Access API unavailable
- IndexedDB (infrastructure)
  - Stores file handle references for File System Access API
  - Enables persistent file access across sessions

**Caching:**
- Service Worker (browser cache)
  - Registration: `public/service-worker.js`
  - Enables offline PWA functionality

## Authentication & Identity

**Auth Provider:**
- None (local-first application)
  - Implementation: No authentication required
  - Data stored locally on user's device

## Monitoring & Observability

**Error Tracking:**
- None (browser console only)

**Logs:**
- Browser console via `web_sys::console` and `log` crate
- Client-side only, no external logging service

## CI/CD & Deployment

**Hosting:**
- Vercel (static site hosting)
  - Config: `public/vercel.json`
  - Environment: Production branch from `main`
  - Headers: COOP/COEP for SharedArrayBuffer support

**CI Pipeline:**
- GitHub Actions
  - Workflows: `.github/workflows/deploy.yml`, `.github/workflows/ci.yml`
  - Build: `devenv shell build` (Dioxus bundle)
  - Release: semantic-release with conventional commits
  - Deployment: Vercel CLI via GitHub Actions

## Environment Configuration

**Required env vars:**
- CI/CD only:
  - `VERCEL_TOKEN` - Vercel deployment authentication
  - `VERCEL_ORG_ID` - Vercel organization identifier
  - `VERCEL_PROJECT_ID` - Vercel project identifier
  - `RELEASE_TOKEN` - GitHub token for semantic-release
  - `GITHUB_TOKEN` - GitHub Actions token

**Secrets location:**
- GitHub repository secrets (for CI/CD)
- No runtime secrets required (client-side only app)

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Browser APIs (Native Integrations)

**File System Access API:**
- Used for: Persistent access to user-selected SQLite database files
- Permissions: User-granted via file picker dialog
- Fallback: LocalStorage when API unavailable
- Implementation: `src/state/file_system.rs`, `public/file-handle-storage.js`

**Web Storage API:**
- LocalStorage: Fallback database storage via `gloo-storage`
- IndexedDB: File handle persistence for File System Access API

**Service Worker API:**
- PWA support: Offline functionality, asset caching
- Registration: `index.html`, implementation in `public/service-worker.js`

**WebAssembly:**
- sql.js: SQLite engine compiled to WASM (`public/sql-wasm.wasm`)
- Dioxus app: Main application compiled to WASM (target: wasm32-unknown-unknown)

---

*Integration audit: 2026-02-25*
