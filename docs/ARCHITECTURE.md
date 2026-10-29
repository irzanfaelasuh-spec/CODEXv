# Architecture

VYRON is a single Rust codebase with native and WASM targets.

- `app/`: application state, lifecycle, screens, panels, commands.
- `ui/`: visual constants and reusable UI primitives.
- `editor/`: tabs, file state, language detection, syntax layouter.
- `workspace/`: real logical workspace data model and file mutations.
- `hierarchy/`: Tree-sitter Rust symbol extraction with a fallback parser for unsupported languages.
- `execution/`: platform-specific execution manager. Native mode uses OS processes; WASM mode only performs browser-safe checks and reports the sandbox limitation honestly.
- `auth/`: local browser datastore for 0.1.0. Passwords are not stored as plaintext, but this is not a replacement for a production auth provider.
- `settings/`: persisted appearance/editor settings.
- `web/`: browser file picker bridge implemented through Rust `web-sys`, not a handwritten JS UI.

The visual frontend is egui rendered through eframe. The browser entrypoint only bootstraps the generated WebAssembly package.
