# Pegasus Zellij Personal Config

This repository keeps Sergio's personal Zellij configuration reproducible. It is intended for static configuration that can be reviewed, copied, linked, and evolved over time.

It is intentionally separate from [`pegasus-zellij-state`](../pegasus-zellij-state), which handles runtime OpenCode/Zellij state integration such as session, tab, and pane title behavior.

## Current slice

The first slice captures a baseline theme named `pegasus-default`. It starts close to the current Zellij look: dark background, light foreground, green active accents, blue/cyan informational accents, and warm warning/error colors.

This is not a final redesign. It is an explicit baseline so future visual changes are deliberate instead of implicit defaults.

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

Use the installer from the repository root. It defaults to `$HOME/.config/zellij` and can be redirected with `ZELLIJ_CONFIG_DIR`.

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

## Publication status

This is a local personal configuration repository. It has no GitHub remote or public repository publication yet.
