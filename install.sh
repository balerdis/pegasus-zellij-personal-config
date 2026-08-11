#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$script_dir

target_dir=${ZELLIJ_CONFIG_DIR:-$HOME/.config/zellij}
target_themes_dir=$target_dir/themes
target_plugins_dir=$target_dir/plugins
backup_root=$target_dir/backups/pegasus-zellij-personal-config
plugin_builder=$repo_root/scripts/build-pegasus-tab-bar.sh
plugin_source=$repo_root/plugins/pegasus-tab-bar/target/wasm32-wasip1/release/pegasus-tab-bar.wasm
permission_granter=$repo_root/scripts/grant-pegasus-tab-bar-permission.py
zellij_cache_dir=${XDG_CACHE_HOME:-$HOME/.cache}/zellij
permissions_cache=$zellij_cache_dir/permissions.kdl

dry_run=0
link_mode=0
backup_dir=""

usage() {
    cat <<'EOF'
Usage: ./install.sh [--dry-run] [--link] [--help]

Install the Pegasus personal Zellij configuration and tab-bar plugin.

Options:
  --dry-run  Print the actions that would be performed without changing files.
  --link     Create symlinks to repository files instead of copying them.
  --help     Show this help message.

Environment:
  ZELLIJ_CONFIG_DIR  Target config directory. Defaults to $HOME/.config/zellij.

Backups:
  Existing target files are backed up before overwrite under:
  $ZELLIJ_CONFIG_DIR/backups/pegasus-zellij-personal-config/<timestamp>
EOF
}

log() {
    printf '%s\n' "$*"
}

run() {
    if [ "$dry_run" -eq 1 ]; then
        log "DRY RUN: $*"
    else
        "$@"
    fi
}

make_backup_dir() {
    if [ -n "$backup_dir" ]; then
        return 0
    fi

    local timestamp candidate suffix
    timestamp=$(date +%Y%m%d-%H%M%S)
    candidate=$backup_root/$timestamp
    suffix=1

    while [ -e "$candidate" ]; do
        candidate=$backup_root/$timestamp-$suffix
        suffix=$((suffix + 1))
    done

    backup_dir=$candidate
    run mkdir -p "$backup_dir"
}

backup_existing() {
    local target relative backup_target
    target=$1

    if [ ! -e "$target" ] && [ ! -L "$target" ]; then
        return 0
    fi

    case "$target" in
        "$backup_root"|"$backup_root"/*)
            log "Skip backup-managed path: $target"
            return 0
            ;;
    esac

    make_backup_dir

    relative=${target#"$target_dir"/}
    backup_target=$backup_dir/$relative
    run mkdir -p "$(dirname -- "$backup_target")"
    run cp -a "$target" "$backup_target"
    log "Backed up: $target -> $backup_target"
}

ensure_parent_dir() {
    local target
    target=$1
    run mkdir -p "$(dirname -- "$target")"
}

install_copy() {
    local source target
    source=$1
    target=$2

    ensure_parent_dir "$target"

    if [ "$dry_run" -eq 1 ]; then
        log "Would copy: $source -> $target"
        return 0
    fi

    if [ -f "$target" ] && cmp -s "$source" "$target"; then
        log "Unchanged: $target"
        return 0
    fi

    backup_existing "$target"
    run cp "$source" "$target"
    log "Copied: $source -> $target"
}

install_link() {
    local source target current_link
    source=$1
    target=$2

    ensure_parent_dir "$target"

    if [ "$dry_run" -eq 1 ]; then
        log "Would link: $target -> $source"
        return 0
    fi

    if [ -L "$target" ]; then
        current_link=$(readlink "$target")
        if [ "$current_link" = "$source" ]; then
            log "Unchanged symlink: $target -> $source"
            return 0
        fi
    fi

    backup_existing "$target"
    run rm -f "$target"
    run ln -s "$source" "$target"
    log "Linked: $target -> $source"
}

install_file() {
    local source target
    source=$1
    target=$2

    if [ "$link_mode" -eq 1 ]; then
        install_link "$source" "$target"
    else
        install_copy "$source" "$target"
    fi
}

read_tab_bar_location() {
    python3 - "$repo_root/config/config.kdl" <<'PY'
import re
import sys
from pathlib import PurePosixPath

config_path = sys.argv[1]
document = open(config_path, encoding="utf-8").read()
matches = re.findall(
    r'^\s*tab-bar\s+location="(file:[^"]+)"\s*(?://.*)?$',
    document,
    flags=re.MULTILINE,
)
if len(matches) != 1:
    raise SystemExit("Expected exactly one tab-bar file: location in config.kdl")

location = matches[0]
raw_path = location.removeprefix("file:")
path = PurePosixPath(raw_path)
if not path.is_absolute() or any(part in {".", ".."} for part in path.parts):
    raise SystemExit(f"Unsafe tab-bar file: location: {location}")

print(location)
print(raw_path)
PY
}

invalidate_plugin_cache() {
    local cache_path
    cache_path=$1

    if [ "$dry_run" -eq 1 ]; then
        log "Would invalidate plugin cache: $cache_path"
        return 0
    fi

    if [ -e "$cache_path" ] || [ -L "$cache_path" ]; then
        rm -rf -- "$cache_path"
        log "Invalidated plugin cache: $cache_path"
    else
        log "Plugin cache already absent: $cache_path"
    fi
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --dry-run)
            dry_run=1
            ;;
        --link)
            link_mode=1
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            printf 'Unknown option: %s\n\n' "$1" >&2
            usage >&2
            exit 1
            ;;
    esac
    shift
done

if [ ! -f "$repo_root/config/config.kdl" ]; then
    printf 'Missing source config: %s\n' "$repo_root/config/config.kdl" >&2
    exit 1
fi

if [ ! -d "$repo_root/themes" ]; then
    printf 'Missing source themes directory: %s\n' "$repo_root/themes" >&2
    exit 1
fi

if [ ! -x "$plugin_builder" ]; then
    printf 'Missing or non-executable plugin builder: %s\n' "$plugin_builder" >&2
    exit 1
fi

if [ ! -x "$permission_granter" ]; then
    printf 'Missing or non-executable permission granter: %s\n' "$permission_granter" >&2
    exit 1
fi

mapfile -t tab_bar_location < <(read_tab_bar_location)
if [ "${#tab_bar_location[@]}" -ne 2 ]; then
    printf 'Could not derive the tab-bar file: location from config.kdl\n' >&2
    exit 1
fi
plugin_location=${tab_bar_location[0]}
plugin_permission_path=${tab_bar_location[1]}
plugin_cache_dir=$zellij_cache_dir/$plugin_location/plugin_cache

mode=copy
if [ "$link_mode" -eq 1 ]; then
    mode=link
fi

log "Pegasus Zellij personal config installer"
log "Repository: $repo_root"
log "Target: $target_dir"
log "Mode: $mode"

if [ "$dry_run" -eq 1 ]; then
    log "DRY RUN: build plugin with $plugin_builder"
    log "DRY RUN: install $plugin_source -> $target_plugins_dir/pegasus-tab-bar.wasm"
else
    "$plugin_builder"
    if [ ! -f "$plugin_source" ]; then
        printf 'Plugin build did not produce: %s\n' "$plugin_source" >&2
        exit 1
    fi
fi

run mkdir -p "$target_dir" "$target_themes_dir" "$target_plugins_dir"

install_file "$repo_root/config/config.kdl" "$target_dir/config.kdl"

while IFS= read -r -d '' theme_file; do
    relative_theme=${theme_file#"$repo_root/themes"/}
    install_file "$theme_file" "$target_themes_dir/$relative_theme"
done < <(find "$repo_root/themes" -type f -name '*.kdl' -print0 | sort -z)

install_file "$plugin_source" "$target_plugins_dir/pegasus-tab-bar.wasm"
invalidate_plugin_cache "$plugin_cache_dir"

if [ "$dry_run" -eq 1 ]; then
    log "Would grant ReadApplicationState only: $plugin_permission_path -> $permissions_cache"
else
    "$permission_granter" --cache "$permissions_cache" --plugin-path "$plugin_permission_path"
fi

if [ -n "$backup_dir" ]; then
    log "Backups: $backup_dir"
else
    log "Backups: none needed"
fi

log "Done. Restart Zellij to load the updated plugin and permission cache."
