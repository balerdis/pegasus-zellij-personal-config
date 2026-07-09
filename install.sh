#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$script_dir

target_dir=${ZELLIJ_CONFIG_DIR:-$HOME/.config/zellij}
target_themes_dir=$target_dir/themes
backup_root=$target_dir/backups/pegasus-zellij-personal-config

dry_run=0
link_mode=0
backup_dir=""

usage() {
    cat <<'EOF'
Usage: ./install.sh [--dry-run] [--link] [--help]

Install the Pegasus personal Zellij configuration.

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

mode=copy
if [ "$link_mode" -eq 1 ]; then
    mode=link
fi

log "Pegasus Zellij personal config installer"
log "Repository: $repo_root"
log "Target: $target_dir"
log "Mode: $mode"

run mkdir -p "$target_dir" "$target_themes_dir"

install_file "$repo_root/config/config.kdl" "$target_dir/config.kdl"

while IFS= read -r -d '' theme_file; do
    relative_theme=${theme_file#"$repo_root/themes"/}
    install_file "$theme_file" "$target_themes_dir/$relative_theme"
done < <(find "$repo_root/themes" -type f -name '*.kdl' -print0 | sort -z)

if [ -n "$backup_dir" ]; then
    log "Backups: $backup_dir"
else
    log "Backups: none needed"
fi

log "Done. Restart Zellij to use updated config or themes."
