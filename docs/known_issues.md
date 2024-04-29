# Known issues

### Unused imports not cleaned for Rust

As of the latest plugin version for Rust, the imports that are unused
are not anymore auto cleaned when pressing `Ctrl+Alt+O`.

To fix this issue, go to `Settings > Editor > Inspections > Rust > Lint`.
Uncheck the `Enable inspection only if procedural macros are enabled`

![Rust clean unused imports settings](./images/rust_auto_clean_imports.png)

## White box glitch

Add the following VM option if you're experiencing white boxes in the UI. This option is enabled by default on the
pre-built versions.

    -Dprism.dirtyopts=false