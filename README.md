# Pegasus Zellij Personal Config

This repository keeps Sergio's personal Zellij configuration reproducible. It includes a focused custom top tab-bar plugin.

It is intentionally separate from [`pegasus-zellij-state`](../pegasus-zellij-state), which handles runtime OpenCode/Zellij state integration such as session, tab, and pane title behavior.

## Current slice

The first slice captures a baseline theme named `pegasus-default`. It starts close to the current Zellij look: dark background, light foreground, green active accents, blue/cyan informational accents, and warm warning/error colors.

This is not a final redesign. It is an explicit baseline so future visual changes are deliberate instead of implicit defaults.

## Custom top tab bar

The `plugins/pegasus-tab-bar` Rust plugin replaces only the built-in `tab-bar` alias. It renders the current session name first, followed by the existing Zellij tab names (including labels supplied by `pegasus-zellij-state`). Active and inactive tabs use the theme's `ribbon_selected` and `ribbon_unselected` colors respectively.

- On load, it requests exactly `ReadApplicationState` and `ChangeApplicationState`. The installer pre-grants those exact permissions for the local Pegasus tab-bar so its non-selectable bar never needs to receive an interactive `y/n` response.
- After approval it subscribes to `ModeUpdate`, `TabUpdate`, and `Mouse`. Only a left click within a rendered tab range switches to that tab; session text, padding, drag, scroll, right click, and keyboard navigation have no plugin side effect.
- Labels are kept verbatim when they fit and otherwise truncated by terminal display width to preserve the one-line bar.

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

### Tab-bar plugin cache

After installing the WASM (in either copy or `--link` mode), the installer
removes only this plugin's cache directory:
`${XDG_CACHE_HOME:-$HOME/.cache}/zellij/file:/…/pegasus-tab-bar.wasm/plugin_cache`.
It derives that path from the exact `tab-bar` `file:` location in
`config/config.kdl`, so unrelated Zellij plugin caches remain intact. Fully
restart Zellij afterward: the running server has already loaded its cached WASM.

### Tab-bar permission cache

The installer writes Zellij's XDG-aware permission cache at
`${XDG_CACHE_HOME:-$HOME/.cache}/zellij/permissions.kdl`. It adds or replaces
only this entry:

```kdl
"/home/serg/.config/zellij/plugins/pegasus-tab-bar.wasm" {
    ReadApplicationState
    ChangeApplicationState
}
```

Although the plugin alias uses
`file:/home/serg/.config/zellij/plugins/pegasus-tab-bar.wasm`, Zellij 0.44.3
serializes file-plugin permission keys as the raw absolute path (without the
`file:` prefix). The installer preserves every unrelated cache entry exactly
and grants no permissions besides `ReadApplicationState` (to render session and
tabs) and `ChangeApplicationState` (to switch only the clicked tab). This is
necessary because the tab bar intentionally remains non-selectable, so Zellij
cannot route the interactive prompt response to it.

If the prompt is already stuck, run `./install.sh --link` and then fully
restart Zellij. The active server reads permissions at startup; dismissing or
typing into the old prompt is not required.

Restart Zellij after changing `config.kdl`, `theme_dir`, theme files, or the
permission cache.

### Plugin target constraint

The committed `tab-bar` alias points to `file:/home/serg/.config/zellij/plugins/pegasus-tab-bar.wasm`, the live default target for this personal configuration. That makes the default `--link` setup work without relying on shell-variable expansion in KDL plugin locations.

When using a non-default `ZELLIJ_CONFIG_DIR`, the installer places the WASM correctly in that target, but the alias remains intentionally pinned to the default live location. Update the alias in `config/config.kdl` to the matching absolute `file:` path before installing such a custom target.

## Publication status

This is a local personal configuration repository. It has no GitHub remote or public repository publication yet.
