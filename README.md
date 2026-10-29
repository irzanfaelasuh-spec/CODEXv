# VYRON IDE 0.1.0

VYRON is a browser-first developer workspace rendered by Rust/egui and compiled to WebAssembly. It is intentionally inspired by the information density and workflow of VS Code without copying VS Code branding or implementation.

## Why this architecture?

VYRON uses **Rust + eframe/egui + WebAssembly**. The application UI is rendered by Rust rather than implemented as a conventional HTML/CSS/JavaScript frontend. eframe officially supports native Windows/macOS/Linux builds and WebAssembly builds. The web build is therefore deployable as static files to GitHub Pages.

## What works in 0.1.0

- 4-second animated VYRON loading screen.
- Local name/password account datastore with auto-login persistence.
- Case-insensitive unique usernames inside the browser datastore.
- Smooth error presentation for duplicate names and invalid credentials.
- VS Code-inspired graphite workspace layout.
- Responsive activity bar, explorer, tabs, editor, status bar and output/terminal areas.
- Real editable workspace content backed by Rust state and browser persistence.
- Save, dirty tabs, create files, rename, delete.
- File upload from the device/browser file manager.
- Native file picker when running the desktop build.
- Syntax-aware editor layouter for common language keywords.
- Rust Tree-sitter hierarchy parsing for real Rust symbols; graceful lexical fallback for other languages.
- Workspace search and in-file find/replace primitives.
- Command palette with Ctrl/Cmd+Shift+P and F5/Ctrl/Cmd+S shortcuts.
- Settings for theme, font size, tab size, wrapping, line numbers, minimap preference, animations and autosave.
- Web console with safe workspace commands: `help`, `ls`, `pwd`, `cat`, `find`, `clear`, `whoami`.
- Native runner for Python, Node.js, shell and a Rust compiler invocation when those runtimes exist on the host.
- Web runner that performs honest browser-safe validation/preview checks and refuses to pretend the browser can spawn an OS compiler.
- Favicon / VYRON mark.
- GitHub Actions workflow that builds the WASM app and deploys to GitHub Pages.

## Important browser limitation

GitHub Pages is a static hosting platform. A static browser app cannot safely provide a server-side global username database or spawn arbitrary OS processes. The 0.1.0 web build therefore uses browser-local persistent storage for accounts and workspace data. That gives auto-login on the same browser/device, but **it does not make usernames globally unique across every user's browser**.

For global accounts, cloud sync, shared projects and a real remote terminal, add a backend (Supabase/Firebase/custom API) in a later release. Do not store production passwords in a custom public database.

## Run locally in the browser

Install Rust stable, the WASM target, and Trunk:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```

Open the printed local URL.

## Native desktop build

```bash
cargo run
cargo test
cargo build --release
```

Native VYRON can use the operating system file picker and real subprocesses for supported runtimes.

## GitHub Pages

1. Create a GitHub repository, e.g. `vyron-ide`.
2. Push this project to the `main` branch.
3. In GitHub: **Settings → Pages → Source: GitHub Actions**.
4. Push to `main` again or run the `Deploy VYRON to GitHub Pages` workflow manually.
5. GitHub will publish the generated `dist/` artifact.

The workflow is already included in `.github/workflows/deploy-pages.yml`.

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| Ctrl/Cmd + S | Save current file |
| Ctrl/Cmd + Shift + P | Command palette |
| F5 | Run current file |
| Enter | Submit login / command palette / terminal input |
| Esc | Close command palette |

## Security notes

The browser version runs inside the browser sandbox. It does not execute arbitrary project binaries. Never treat uploaded or untrusted source code as trusted. The native build executes programs with the permissions of the user account running VYRON.

## Roadmap

0.2.0: remote authentication + cloud workspace adapter, multi-file upload, stronger syntax highlighting, Git status/diff UI, real browser terminal via a remote agent.

0.3.0: LSP client, diagnostics, richer completion, DAP-backed debugging, workspace indexing, extension API.
