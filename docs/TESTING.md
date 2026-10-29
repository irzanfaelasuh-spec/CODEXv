# Testing

## Environment available to the packaging run

The packaging environment did not have `cargo` or `rustc` installed, so a Rust compilation or graphical execution test could not be honestly performed here.

## Tests included

- Duplicate usernames are rejected case-insensitively.
- Workspace files can be added and renamed.
- Rust hierarchy extraction identifies struct and function symbols.

Run them with:

```bash
cargo test
```

## Release verification checklist

After cloning on a Rust-enabled machine:

1. `cargo test`
2. `trunk build --release`
3. Open `dist/index.html` through a static server.
4. Wait for the 4-second splash.
5. Create a user and close/reopen the browser; verify auto-login.
6. Create a file, edit it, save it, reload, and verify the content remains.
7. Upload a file from the device file manager.
8. Verify the Rust Outline updates and points to real line numbers.
9. Run a JSON file in web mode and verify validation output.
10. On native builds, run a small Python/Node/Rust program if the corresponding runtime is installed.
11. Stop a native long-running process and verify state cleanup.

No unexecuted test is marked as passed by this repository.
