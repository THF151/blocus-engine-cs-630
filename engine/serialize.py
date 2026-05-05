#!/usr/bin/env python3
"""
Serialize the current directory recursively into a clean Markdown document
and copy the result to the system clipboard.

Features:
- Prints a directory tree first.
- Serializes text/code files with correct Markdown code fences.
- Converts Jupyter notebooks into readable Markdown instead of raw JSON.
- Excludes large files, binary files, generated/build/cache/vendor directories,
  media/archive/binary formats, and noisy lock/cache files.
- Skips files with more than 5000 source lines.
- Skips files whose serialized representation would exceed 4000 lines.
- Copies the final Markdown to the clipboard.
- Prints character count, serialized file count, excluded file count, and
  approximate LLM token count.
- Uses only the Python standard library.

Usage:
    python serialize_project.py
    python serialize_project.py --root /path/to/project
    python serialize_project.py --print-preview
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Optional


MAX_SOURCE_LINES_DEFAULT = 5000
MAX_SERIALIZED_LINES_DEFAULT = 4000
MAX_BYTES_DEFAULT = 2_000_000


EXCLUDED_DIR_NAMES = {
    # Python
    ".venv",
    "venv",
    "tests",
    "env",
    ".env",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".nox",
    ".coverage",
    "htmlcov",
    ".eggs",
    "*.egg-info",

    # JavaScript / TypeScript / frontend
    "node_modules",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".astro",
    ".angular",
    "coverage",
    ".nyc_output",

    # Build / distribution
    "build",
    "dist",
    "target",
    "out",
    "bin",
    "obj",
    ".gradle",
    ".mvn",
    ".parcel-cache",
    ".turbo",
    ".cache",

    # IDE / editor
    ".idea",
    ".vscode",
    ".fleet",
    ".zed",

    # Version control
    ".git",
    ".svn",
    ".hg",

    # OS / misc
    ".DS_Store",
    "Thumbs.db",

    # Rust / Go / Java / JVM
    "vendor",
    ".cargo",
    ".stack-work",
    ".bloop",
    ".metals",

    # Data science / notebooks
    ".ipynb_checkpoints",

    # Terraform / infra
    ".terraform",
    ".serverless",
    ".aws-sam",

    # Mobile
    "Pods",
    "DerivedData",
    ".expo",
}


EXCLUDED_FILE_NAMES = {
    # OS/editor noise
    ".DS_Store",
    "Thumbs.db",
    "desktop.ini",

    # Environment / secrets
    ".env",
    ".env.local",
    ".env.development",
    ".env.production",
    ".env.test",
    ".npmrc",
    ".pypirc",
    ".netrc",

    # Lock files that are often huge/noisy
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "poetry.lock",
    "Pipfile.lock",
    "Cargo.lock",
    "composer.lock",
    "Gemfile.lock",
    "go.sum",

    # Generated metadata
    ".coverage",
}


EXCLUDED_SUFFIXES = {
    # Images
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".tiff", ".tif",
    ".ico", ".icns", ".svgz", ".heic", ".avif",

    # Audio
    ".mp3", ".wav", ".flac", ".aac", ".ogg", ".m4a", ".wma",

    # Video
    ".mp4", ".mov", ".avi", ".mkv", ".webm", ".wmv", ".m4v",

    # Archives / compressed
    ".zip", ".tar", ".gz", ".tgz", ".bz2", ".xz", ".7z", ".rar", ".zst",

    # Documents / binary office formats
    ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
    ".odt", ".ods", ".odp",

    # Binaries / executables / libraries
    ".exe", ".dll", ".so", ".dylib", ".bin", ".dat", ".o", ".a",
    ".class", ".jar", ".war", ".ear", ".pyc", ".pyo", ".pyd",

    # Fonts
    ".ttf", ".otf", ".woff", ".woff2", ".eot",

    # Database / local storage
    ".sqlite", ".sqlite3", ".db", ".mdb", ".accdb",

    # Model / large data artifacts
    ".h5", ".hdf5", ".onnx", ".pt", ".pth", ".ckpt", ".safetensors",
    ".parquet", ".feather", ".arrow", ".orc",

    # Misc generated artifacts
    ".lockb", ".map",
}


LANGUAGE_BY_SUFFIX = {
    ".py": "python",
    ".pyw": "python",
    ".js": "javascript",
    ".jsx": "jsx",
    ".ts": "typescript",
    ".tsx": "tsx",
    ".html": "html",
    ".htm": "html",
    ".css": "css",
    ".scss": "scss",
    ".sass": "sass",
    ".less": "less",
    ".json": "json",
    ".jsonc": "jsonc",
    ".yaml": "yaml",
    ".yml": "yaml",
    ".toml": "toml",
    ".ini": "ini",
    ".cfg": "ini",
    ".conf": "conf",
    ".md": "markdown",
    ".mdx": "mdx",
    ".rst": "rst",
    ".sh": "bash",
    ".bash": "bash",
    ".zsh": "zsh",
    ".fish": "fish",
    ".ps1": "powershell",
    ".bat": "batch",
    ".cmd": "batch",
    ".sql": "sql",
    ".xml": "xml",
    ".xsd": "xml",
    ".java": "java",
    ".kt": "kotlin",
    ".kts": "kotlin",
    ".scala": "scala",
    ".go": "go",
    ".rs": "rust",
    ".c": "c",
    ".h": "c",
    ".cpp": "cpp",
    ".cc": "cpp",
    ".cxx": "cpp",
    ".hpp": "cpp",
    ".cs": "csharp",
    ".php": "php",
    ".rb": "ruby",
    ".swift": "swift",
    ".dart": "dart",
    ".lua": "lua",
    ".r": "r",
    ".R": "r",
    ".m": "objective-c",
    ".mm": "objective-cpp",
    ".tex": "tex",
    ".dockerfile": "dockerfile",
    ".graphql": "graphql",
    ".gql": "graphql",
    ".proto": "protobuf",
    ".vue": "vue",
    ".svelte": "svelte",
}


LANGUAGE_BY_NAME = {
    "Dockerfile": "dockerfile",
    "dockerfile": "dockerfile",
    "Makefile": "makefile",
    "makefile": "makefile",
    "CMakeLists.txt": "cmake",
    ".gitignore": "gitignore",
    ".dockerignore": "gitignore",
    ".editorconfig": "editorconfig",
    ".prettierrc": "json",
    ".eslintrc": "json",
    ".babelrc": "json",
}


@dataclass(frozen=True)
class Config:
    root: Path
    max_source_lines: int
    max_serialized_lines: int
    max_bytes: int
    include_hidden: bool
    print_preview: bool


@dataclass
class FileResult:
    path: Path
    relative_path: Path
    serialized: Optional[str]
    included: bool
    reason: Optional[str] = None


def parse_args() -> Config:
    parser = argparse.ArgumentParser(
        description="Serialize a project directory into Markdown and copy it to the clipboard."
    )
    parser.add_argument(
        "--root",
        default=".",
        help="Root directory to serialize. Defaults to current directory.",
    )
    parser.add_argument(
        "--max-source-lines",
        type=int,
        default=MAX_SOURCE_LINES_DEFAULT,
        help="Skip files with more than this many source lines.",
    )
    parser.add_argument(
        "--max-serialized-lines",
        type=int,
        default=MAX_SERIALIZED_LINES_DEFAULT,
        help="Skip files whose Markdown serialization exceeds this many lines.",
    )
    parser.add_argument(
        "--max-bytes",
        type=int,
        default=MAX_BYTES_DEFAULT,
        help="Skip files larger than this many bytes.",
    )
    parser.add_argument(
        "--include-hidden",
        action="store_true",
        help="Include hidden files/directories unless otherwise excluded.",
    )
    parser.add_argument(
        "--print-preview",
        action="store_true",
        help="Also print the generated Markdown to stdout.",
    )

    args = parser.parse_args()

    return Config(
        root=Path(args.root).resolve(),
        max_source_lines=args.max_source_lines,
        max_serialized_lines=args.max_serialized_lines,
        max_bytes=args.max_bytes,
        include_hidden=args.include_hidden,
        print_preview=args.print_preview,
    )


def wildcard_dir_excluded(name: str) -> bool:
    for pattern in EXCLUDED_DIR_NAMES:
        if "*" in pattern:
            regex = "^" + re.escape(pattern).replace("\\*", ".*") + "$"
            if re.match(regex, name):
                return True
    return False


def should_exclude_dir(path: Path, config: Config) -> Optional[str]:
    name = path.name

    if name in EXCLUDED_DIR_NAMES or wildcard_dir_excluded(name):
        return f"excluded directory: {name}"

    if not config.include_hidden and name.startswith("."):
        return f"hidden directory: {name}"

    return None


def should_exclude_file(path: Path, config: Config) -> Optional[str]:
    name = path.name
    suffix = path.suffix.lower()

    if name in EXCLUDED_FILE_NAMES:
        return f"excluded file name: {name}"

    if suffix in EXCLUDED_SUFFIXES:
        return f"excluded file type: {suffix}"

    if not config.include_hidden and name.startswith(".") and name not in LANGUAGE_BY_NAME:
        return f"hidden file: {name}"

    try:
        size = path.stat().st_size
    except OSError as exc:
        return f"cannot stat file: {exc}"

    if size > config.max_bytes:
        return f"file too large: {size} bytes"

    return None


def is_probably_binary(path: Path, sample_size: int = 8192) -> bool:
    try:
        chunk = path.read_bytes()[:sample_size]
    except OSError:
        return True

    if not chunk:
        return False

    if b"\x00" in chunk:
        return True

    text_characters = bytearray({7, 8, 9, 10, 12, 13, 27} | set(range(32, 127)))
    non_text = chunk.translate(None, text_characters)

    return len(non_text) / len(chunk) > 0.30


def read_text(path: Path) -> Optional[str]:
    for encoding in ("utf-8", "utf-8-sig", "latin-1"):
        try:
            return path.read_text(encoding=encoding)
        except UnicodeDecodeError:
            continue
        except OSError:
            return None

    return None


def count_lines(text: str) -> int:
    if not text:
        return 0
    return text.count("\n") + 1


def estimate_llm_tokens(text: str) -> int:
    """
    Approximate token count.

    This intentionally avoids external dependencies. For English/code-heavy text,
    characters / 4 is a common rough estimate. The regex-based word/punctuation
    estimate helps avoid severe undercounting for code and symbol-heavy content.
    """
    char_estimate = len(text) / 4
    lexical_estimate = len(re.findall(r"\w+|[^\w\s]", text, flags=re.UNICODE))

    return int(max(char_estimate, lexical_estimate))


def language_for_file(path: Path) -> str:
    if path.name in LANGUAGE_BY_NAME:
        return LANGUAGE_BY_NAME[path.name]

    if path.suffix in LANGUAGE_BY_SUFFIX:
        return LANGUAGE_BY_SUFFIX[path.suffix]

    lower_suffix = path.suffix.lower()
    if lower_suffix in LANGUAGE_BY_SUFFIX:
        return LANGUAGE_BY_SUFFIX[lower_suffix]

    return ""


def fence_for_content(content: str) -> str:
    longest = 0
    for match in re.finditer(r"`+", content):
        longest = max(longest, len(match.group(0)))

    return "`" * max(3, longest + 1)


def markdown_code_block(content: str, language: str = "") -> str:
    fence = fence_for_content(content)
    lang = language.strip()
    return f"{fence}{lang}\n{content.rstrip()}\n{fence}"


def notebook_cell_source(cell: dict) -> str:
    source = cell.get("source", "")
    if isinstance(source, list):
        return "".join(str(part) for part in source)
    return str(source)


def notebook_outputs_to_markdown(cell: dict) -> str:
    outputs = cell.get("outputs") or []
    rendered: list[str] = []

    for output in outputs:
        output_type = output.get("output_type")

        if output_type == "stream":
            text = output.get("text", "")
            if isinstance(text, list):
                text = "".join(str(part) for part in text)
            if text.strip():
                rendered.append("**Output:**\n\n" + markdown_code_block(str(text), ""))

        elif output_type in {"execute_result", "display_data"}:
            data = output.get("data", {})

            if "text/markdown" in data:
                md = data["text/markdown"]
                if isinstance(md, list):
                    md = "".join(str(part) for part in md)
                rendered.append(str(md).rstrip())

            elif "text/plain" in data:
                plain = data["text/plain"]
                if isinstance(plain, list):
                    plain = "".join(str(part) for part in plain)
                if str(plain).strip():
                    rendered.append("**Output:**\n\n" + markdown_code_block(str(plain), ""))

            elif any(key.startswith("image/") for key in data):
                rendered.append("**Output:** image data omitted.")

            else:
                rendered.append("**Output:** non-text display data omitted.")

        elif output_type == "error":
            traceback = output.get("traceback", [])
            if isinstance(traceback, list):
                traceback_text = "\n".join(str(line) for line in traceback)
            else:
                traceback_text = str(traceback)

            if traceback_text.strip():
                rendered.append("**Error:**\n\n" + markdown_code_block(traceback_text, ""))

    return "\n\n".join(rendered).strip()


def serialize_notebook(path: Path) -> Optional[str]:
    text = read_text(path)
    if text is None:
        return None

    try:
        notebook = json.loads(text)
    except json.JSONDecodeError:
        return None

    cells = notebook.get("cells", [])
    if not isinstance(cells, list):
        return None

    parts: list[str] = []

    for index, cell in enumerate(cells, start=1):
        if not isinstance(cell, dict):
            continue

        cell_type = cell.get("cell_type")
        source = notebook_cell_source(cell).rstrip()

        if cell_type == "markdown":
            if source:
                parts.append(source)

        elif cell_type == "code":
            execution_count = cell.get("execution_count")
            label = f"Code cell {index}"
            if execution_count is not None:
                label += f" / execution_count={execution_count}"

            block = f"#### {label}\n\n"
            block += markdown_code_block(source, "python") if source else "_Empty code cell._"

            outputs_md = notebook_outputs_to_markdown(cell)
            if outputs_md:
                block += "\n\n" + outputs_md

            parts.append(block)

        elif source:
            parts.append(f"#### {cell_type or 'unknown'} cell {index}\n\n{source}")

    return "\n\n".join(parts).strip() + "\n"


def serialize_regular_file(path: Path) -> Optional[str]:
    text = read_text(path)
    if text is None:
        return None

    language = language_for_file(path)
    return markdown_code_block(text, language) + "\n"


def serialize_file(path: Path, config: Config) -> FileResult:
    relative = path.relative_to(config.root)

    exclusion_reason = should_exclude_file(path, config)
    if exclusion_reason:
        return FileResult(path, relative, None, False, exclusion_reason)

    if is_probably_binary(path):
        return FileResult(path, relative, None, False, "binary or non-text data")

    raw_text = read_text(path)
    if raw_text is None:
        return FileResult(path, relative, None, False, "unreadable text")

    source_lines = count_lines(raw_text)
    if source_lines > config.max_source_lines:
        return FileResult(
            path,
            relative,
            None,
            False,
            f"too many source lines: {source_lines}",
        )

    if path.suffix.lower() == ".ipynb":
        serialized = serialize_notebook(path)
        if serialized is None:
            return FileResult(path, relative, None, False, "invalid notebook")
    else:
        serialized = serialize_regular_file(path)
        if serialized is None:
            return FileResult(path, relative, None, False, "unserializable text")

    serialized_lines = count_lines(serialized)
    if serialized_lines > config.max_serialized_lines:
        return FileResult(
            path,
            relative,
            None,
            False,
            f"serialized output too long: {serialized_lines} lines",
        )

    return FileResult(path, relative, serialized, True)


def iter_files(config: Config) -> Iterable[Path]:
    for current_root, dirnames, filenames in os.walk(config.root):
        current_path = Path(current_root)

        kept_dirs = []
        for dirname in sorted(dirnames):
            dir_path = current_path / dirname
            reason = should_exclude_dir(dir_path, config)
            if reason is None:
                kept_dirs.append(dirname)

        dirnames[:] = kept_dirs

        for filename in sorted(filenames):
            yield current_path / filename


def collect_results(config: Config) -> list[FileResult]:
    results = []
    for path in iter_files(config):
        if path.is_file():
            results.append(serialize_file(path, config))
    return results


def build_tree(results: list[FileResult]) -> str:
    included_paths = sorted(result.relative_path for result in results if result.included)

    if not included_paths:
        return "_No files included._"

    tree: dict[str, dict] = {}

    for relative_path in included_paths:
        node = tree
        for part in relative_path.parts:
            node = node.setdefault(part, {})

    lines: list[str] = []

    def walk(node: dict, prefix: str = "") -> None:
        entries = sorted(node.items(), key=lambda item: (bool(item[1]), item[0].lower()))

        for index, (name, child) in enumerate(entries):
            connector = "└── " if index == len(entries) - 1 else "├── "
            lines.append(prefix + connector + name)

            if child:
                extension = "    " if index == len(entries) - 1 else "│   "
                walk(child, prefix + extension)

    lines.append(".")
    walk(tree)

    return "\n".join(lines)


def markdown_heading_for_path(relative_path: Path) -> str:
    return "## " + str(relative_path).replace("\\", "/")


def build_markdown(config: Config, results: list[FileResult]) -> str:
    included = [result for result in results if result.included]
    excluded = [result for result in results if not result.included]

    lines: list[str] = [
        "# Project Serialization",
        "",
        f"Root: `{config.root}`",
        "",
        "## Directory Structure",
        "",
        "```text",
        build_tree(results),
        "```",
        "",
        "## Serialization Settings",
        "",
        f"- Maximum source lines per file: `{config.max_source_lines}`",
        f"- Maximum serialized lines per file: `{config.max_serialized_lines}`",
        f"- Maximum bytes per file: `{config.max_bytes}`",
        f"- Included files: `{len(included)}`",
        f"- Excluded files: `{len(excluded)}`",
        "",
    ]

    if excluded:
        lines.extend([
            "## Excluded Files",
            "",
            "| File | Reason |",
            "|---|---|",
        ])

        for result in sorted(excluded, key=lambda r: str(r.relative_path)):
            file_name = str(result.relative_path).replace("\\", "/")
            reason = result.reason or "excluded"
            lines.append(f"| `{file_name}` | {reason} |")

        lines.append("")

    lines.append("# Serialized Files")
    lines.append("")

    for result in sorted(included, key=lambda r: str(r.relative_path)):
        lines.append(markdown_heading_for_path(result.relative_path))
        lines.append("")
        lines.append(result.serialized or "")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def copy_to_clipboard_macos(text: str) -> Optional[str]:
    if shutil.which("pbcopy") is None:
        return None

    subprocess.run(
        ["pbcopy"],
        input=text,
        text=True,
        check=True,
    )
    return "pbcopy"


def copy_to_clipboard_windows(text: str) -> Optional[str]:
    if shutil.which("clip") is None:
        return None

    subprocess.run(
        ["clip"],
        input=text,
        text=True,
        check=True,
    )
    return "clip"


def copy_to_clipboard_linux(text: str) -> Optional[str]:
    commands = [
        ("wl-copy", ["wl-copy"]),
        ("xclip", ["xclip", "-selection", "clipboard"]),
        ("xsel", ["xsel", "--clipboard", "--input"]),
    ]

    for backend, command in commands:
        if shutil.which(command[0]) is None:
            continue

        subprocess.run(
            command,
            input=text,
            text=True,
            check=True,
        )
        return backend

    return None


def copy_to_clipboard_tkinter(text: str) -> Optional[str]:
    try:
        import tkinter as tk
    except Exception:
        return None

    try:
        root = tk.Tk()
        root.withdraw()
        root.clipboard_clear()
        root.clipboard_append(text)
        root.update()
        root.destroy()
        return "tkinter"
    except Exception:
        return None


def copy_to_clipboard(text: str) -> str:
    system = platform.system().lower()

    attempts = []

    if system == "darwin":
        attempts = [copy_to_clipboard_macos, copy_to_clipboard_tkinter]
    elif system == "windows":
        attempts = [copy_to_clipboard_windows, copy_to_clipboard_tkinter]
    else:
        attempts = [copy_to_clipboard_linux, copy_to_clipboard_tkinter]

    errors: list[str] = []

    for attempt in attempts:
        try:
            backend = attempt(text)
            if backend:
                return backend
        except Exception as exc:
            errors.append(f"{attempt.__name__}: {exc}")

    error_text = "\n".join(errors) if errors else "No clipboard backend available."
    raise RuntimeError(
        "Could not copy to clipboard.\n"
        "Install one of the following clipboard tools if needed:\n"
        "- Linux Wayland: wl-copy\n"
        "- Linux X11: xclip or xsel\n"
        "- macOS: pbcopy should already be available\n"
        "- Windows: clip should already be available\n\n"
        f"Details:\n{error_text}"
    )


def main() -> int:
    config = parse_args()

    if not config.root.exists():
        print(f"Root does not exist: {config.root}", file=sys.stderr)
        return 1

    if not config.root.is_dir():
        print(f"Root is not a directory: {config.root}", file=sys.stderr)
        return 1

    results = collect_results(config)
    markdown = build_markdown(config, results)

    included_count = sum(1 for result in results if result.included)
    excluded_count = sum(1 for result in results if not result.included)
    character_count = len(markdown)
    line_count = count_lines(markdown)
    estimated_tokens = estimate_llm_tokens(markdown)

    try:
        clipboard_backend = copy_to_clipboard(markdown)
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    if config.print_preview:
        print(markdown)

    print("Project serialized and copied to clipboard.")
    print(f"Root: {config.root}")
    print(f"Serialized files: {included_count}")
    print(f"Excluded files: {excluded_count}")
    print(f"Characters: {character_count:,}")
    print(f"Lines: {line_count:,}")
    print(f"Estimated LLM tokens: {estimated_tokens:,}")
    print(f"Clipboard backend: {clipboard_backend}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())