#!/usr/bin/env python3
"""Grant the Pegasus tab bar its exact required Zellij permissions.

This deliberately edits only this plugin's top-level KDL node, leaving every
other cache entry byte-for-byte intact. ReadApplicationState renders the current
session and tabs; ChangeApplicationState is required only to switch to a clicked
tab.
"""

from __future__ import annotations

import argparse
import os
import stat
import tempfile
from pathlib import Path


DEFAULT_PLUGIN_PATH = "/home/serg/.config/zellij/plugins/pegasus-tab-bar.wasm"


def matching_brace(document: str, opening_brace: int) -> int:
    """Return the matching brace while ignoring KDL strings and comments."""
    depth = 0
    index = opening_brace
    in_string = False
    escaped = False
    line_comment = False
    block_comment_depth = 0

    while index < len(document):
        char = document[index]
        next_char = document[index + 1] if index + 1 < len(document) else ""

        if line_comment:
            if char in "\r\n":
                line_comment = False
        elif block_comment_depth:
            if char == "/" and next_char == "*":
                block_comment_depth += 1
                index += 1
            elif char == "*" and next_char == "/":
                block_comment_depth -= 1
                index += 1
        elif in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
        elif char == "/" and next_char == "/":
            line_comment = True
            index += 1
        elif char == "/" and next_char == "*":
            block_comment_depth = 1
            index += 1
        elif char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
        index += 1

    raise ValueError("unterminated Pegasus tab-bar permission node")


def node_ranges(document: str, plugin_path: str) -> list[tuple[int, int]]:
    """Find top-level quoted nodes for the known local plugin path."""
    needle = f'"{plugin_path}"'
    ranges: list[tuple[int, int]] = []
    start = 0

    while True:
        name_start = document.find(needle, start)
        if name_start == -1:
            return ranges

        line_start = document.rfind("\n", 0, name_start) + 1
        # Zellij's serializer writes each cache key as a root-level quoted node.
        # Reject occurrences in comments, strings, or inline values instead of
        # risking an edit to an unrelated entry.
        if document[line_start:name_start].strip():
            start = name_start + len(needle)
            continue
        cursor = name_start + len(needle)
        while cursor < len(document) and document[cursor] in " \t\r":
            cursor += 1
        if cursor < len(document) and document[cursor] == "{":
            node_end = matching_brace(document, cursor) + 1
            # The canonical replacement includes its final newline. Consume one
            # existing newline so repeated runs do not accumulate blank lines.
            if document[node_end:node_end + 2] == "\r\n":
                node_end += 2
            elif document[node_end:node_end + 1] == "\n":
                node_end += 1
            ranges.append((line_start, node_end))
        start = name_start + len(needle)


def permission_node(plugin_path: str) -> str:
    return (
        f'"{plugin_path}" {{\n'
        "    ReadApplicationState\n"
        "    ChangeApplicationState\n"
        "}\n"
    )


def update_cache(document: str, plugin_path: str) -> str:
    """Replace only this plugin's nodes, or append its canonical one."""
    ranges = node_ranges(document, plugin_path)
    node = permission_node(plugin_path)
    if not ranges:
        separator = "" if not document or document.endswith("\n") else "\n"
        return f"{document}{separator}{node}"

    updated = document
    for start, end in reversed(ranges):
        updated = f"{updated[:start]}{node}{updated[end:]}"
    return updated


def write_atomically(cache_path: Path, content: str) -> None:
    cache_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    mode = stat.S_IMODE(cache_path.stat().st_mode) if cache_path.exists() else 0o600
    fd, temporary_path = tempfile.mkstemp(prefix=".permissions.", dir=cache_path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as temporary_file:
            temporary_file.write(content)
        os.chmod(temporary_path, mode)
        os.replace(temporary_path, cache_path)
    except BaseException:
        os.unlink(temporary_path)
        raise


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cache", required=True, type=Path)
    parser.add_argument("--plugin-path", default=DEFAULT_PLUGIN_PATH)
    args = parser.parse_args()

    original = args.cache.read_text(encoding="utf-8") if args.cache.exists() else ""
    updated = update_cache(original, args.plugin_path)
    if updated != original:
        write_atomically(args.cache, updated)
        print(f"Granted ReadApplicationState and ChangeApplicationState: {args.plugin_path}")
    else:
        print(f"Permission already exact: {args.plugin_path}")


if __name__ == "__main__":
    main()
