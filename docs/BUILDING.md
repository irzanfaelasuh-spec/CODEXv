# Building VYRON

## Prerequisites

- Rust stable
- `rustup target add wasm32-unknown-unknown`
- Trunk for the web build
- A modern Chrome/Edge/Firefox/Safari for the web build
- Optional native runtimes: Python, Node.js, Rust toolchain, shell

## Web development

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```

## Web release

```bash
trunk build --release
```

Upload the generated `dist/` directory to any static host. The repository includes GitHub Pages Actions deployment.

## Native

```bash
cargo test
cargo run
cargo build --release
```

### Linux notes

Depending on the graphics backend, egui/eframe may require X11/Wayland and OpenGL/Vulkan development packages. The exact packages vary by distribution.

### Windows notes

Install the Visual Studio C++ build tools or the required Rust MSVC toolchain. The native file picker uses `rfd`.

### macOS notes

Install Xcode Command Line Tools and a Rust target appropriate for the machine.

## Troubleshooting

- If `trunk` is missing, install it with `cargo install trunk`.
- If WASM linking fails, run `rustup target add wasm32-unknown-unknown`.
- If a native run command fails, verify the corresponding runtime is on `PATH`.
- Browser file upload requires a browser that supports standard file input APIs.
