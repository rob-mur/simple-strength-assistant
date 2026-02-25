# Technology Stack

**Analysis Date:** 2026-02-25

## Languages

**Primary:**
- Rust 1.92.0 (2024 edition) - Core application logic, WASM compilation
- JavaScript (ES Modules) - Database bridge, File System Access API integration

**Secondary:**
- CSS - Styling via Tailwind/PostCSS

## Runtime

**Environment:**
- WebAssembly (wasm32-unknown-unknown target)
- Browser-based (Web platform)

**Package Manager:**
- Cargo (Rust) - `Cargo.lock` present
- npm (Node.js v22.21.1) - `package-lock.json` present
- Lockfiles: Both present and committed

## Frameworks

**Core:**
- Dioxus 0.7.0 - Rust web framework (React-like for Rust/WASM)
- Tailwind CSS 3.4.17 - Utility-first CSS framework
- DaisyUI 4.12.14 - Tailwind component library

**Testing:**
- wasm-bindgen-test 0.3 - WASM testing framework

**Build/Dev:**
- dioxus-cli - Build and dev server for Dioxus apps
- wasm-bindgen 0.2.106 - Rust-JS interop
- wasm-bindgen-cli - WASM bindings generation
- binaryen - WebAssembly optimization (wasm-opt)
- PostCSS 8.4.49 - CSS processing
- Autoprefixer 10.4.20 - CSS vendor prefixing
- tsx - TypeScript execution for scripts
- devenv (Nix-based) - Development environment management

## Key Dependencies

**Critical:**
- sql.js (via `public/sql-wasm.js` and `public/sql-wasm.wasm`) - SQLite compiled to WASM for in-browser database
- web-sys 0.3 - Web API bindings for Rust (Window, Navigator, FileSystemHandle, etc.)
- js-sys 0.3 - JavaScript standard types for Rust
- wasm-bindgen-futures 0.4 - Async bridge between Rust futures and JS Promises

**Infrastructure:**
- serde 1.0 - Serialization/deserialization
- serde_json 1.0 - JSON support
- gloo-storage 0.3 - Web Storage API wrapper (LocalStorage)
- gloo-utils 0.2 - General utilities for WASM
- thiserror 2.0 - Error handling ergonomics
- log 0.4 - Logging facade

**Release/CI:**
- semantic-release 24.2.0 - Automated versioning and releases
- @commitlint/cli 19.6.1 - Commit message linting
- conventional-changelog-conventionalcommits 8.0.0 - Conventional commit support

**Deployment:**
- vercel 50.4.5 - Deployment CLI for Vercel platform

## Configuration

**Environment:**
- `.envrc` present (direnv for local environment)
- Configuration via `devenv.nix` and `devenv.yaml`
- No `.env` files detected in root (environment config likely in CI/deployment)

**Build:**
- `Dioxus.toml` - Dioxus application configuration
- `Cargo.toml` - Rust dependencies and project metadata
- `package.json` - Node/npm scripts for CSS processing
- `tailwind.config.js` - Tailwind configuration
- `postcss.config.js` - PostCSS plugins
- `.releaserc.json` - Semantic release configuration

## Platform Requirements

**Development:**
- Rust stable with wasm32-unknown-unknown target
- Node.js 22.x
- Nix (optional, for devenv)
- dioxus-cli for local dev server
- wasm-bindgen-cli for bindings

**Production:**
- Modern browser with:
  - WebAssembly support
  - File System Access API (optional, fallback to IndexedDB)
  - Service Worker support
  - LocalStorage/IndexedDB
  - Cross-Origin-Embedder-Policy headers for SharedArrayBuffer

---

*Stack analysis: 2026-02-25*
