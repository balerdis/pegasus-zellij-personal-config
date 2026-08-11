# Pegasus Zellij Personal Config

This repository keeps Sergio's personal Zellij configuration reproducible. It includes a focused custom top tab-bar plugin.

It is intentionally separate from [`pegasus-zellij-state`](../pegasus-zellij-state), which handles runtime OpenCode/Zellij state integration such as session, tab, and pane title behavior.

## Current slice

The first slice captures a baseline theme named `pegasus-default`. It starts close to the current Zellij look: dark background, light foreground, green active accents, blue/cyan informational accents, and warm warning/error colors.

This is not a final redesign. It is an explicit baseline so future visual changes are deliberate instead of implicit defaults.

## Custom top tab bar

The `plugins/pegasus-tab-bar` Rust plugin replaces only the built-in `tab-bar` alias. The current safety baseline renders:

```text
Pegasus tab bar
```

- It uses no application-state permissions, ANSI styling, or mouse handling.
- The dynamic session name, tab labels, and click-to-focus layer is intentionally deferred until it is tested against a live permission prompt in an isolated Zellij session.

`pegasus-zellij-state` remains independent: it owns the OpenCode-to-Zellij runtime updates that produce labels such as `OC | working (1)`. This repository only displays those existing names.

## Rust prerequisite

The installer builds the plugin locally. It requires Rust, Cargo, and the `wasm32-wasip1` target. Zellij 0.44.3 documents the older target name `wasm32-wasi`; current Rust toolchains provide its WASI Preview 1 replacement as `wasm32-wasip1`, which is the target used by this repository.

If Rust is not installed, use the standard Rustup installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The build helper exposes `$HOME/.cargo/bin` for its own process and adds the required target when absent:

```bash
./scripts/build-pegasus-tab-bar.sh
```

It prints the deterministic artifact path:

```text
plugins/pegasus-tab-bar/target/wasm32-wasip1/release/pegasus-tab-bar.wasm
```

## Planned layers

- Themes
- `config.kdl`
- Layouts
- Plugins/status bar
- Keybindings
- Installer/applier (first safe installer is available in `install.sh`)

## Repository layout

```text
.
├── config/
│   └── config.kdl
├── install.sh
├── themes/
│   └── pegasus-default.kdl
├── .gitignore
└── README.md
```

## Install or apply

Use the installer from the repository root. It defaults to `$HOME/.config/zellij` and can be redirected with `ZELLIJ_CONFIG_DIR`. A normal install builds the WASM plugin first, then installs `plugins/pegasus-tab-bar.wasm` with the same copy or link mode as the config and themes.

Preview the actions first:

```bash
./install.sh --dry-run
```

Copy the config and themes:

```bash
./install.sh
```

Or symlink the installed files back to this repository:

```bash
./install.sh --link
```

Existing target files are backed up before overwrite under:

```text
$ZELLIJ_CONFIG_DIR/backups/pegasus-zellij-personal-config/<timestamp>
```

The installer does not delete the backups directory. Repeated runs skip unchanged copied files and already-correct symlinks.

Restart Zellij after changing `config.kdl`, `theme_dir`, or theme files.

### Plugin target constraint

The committed `tab-bar` alias points to `file:/home/serg/.config/zellij/plugins/pegasus-tab-bar.wasm`, the live default target for this personal configuration. That makes the default `--link` setup work without relying on shell-variable expansion in KDL plugin locations.

When using a non-default `ZELLIJ_CONFIG_DIR`, the installer places the WASM correctly in that target, but the alias remains intentionally pinned to the default live location. Update the alias in `config/config.kdl` to the matching absolute `file:` path before installing such a custom target.

## Publication status

This is a local personal configuration repository. It has no GitHub remote or public repository publication yet.
