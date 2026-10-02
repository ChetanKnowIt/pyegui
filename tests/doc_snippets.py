"""Check the Python snippets in the docs still match the code.

The examples job runs every app in `examples/`, `guides/` and `debug.py`. None
of them check the documentation, so a snippet in README.rst can show an API
that no longer exists and every job stays green -- which is exactly what
happened to the two snippets that used to call `run_native` without a
container.

Each runnable snippet is compiled, and the ones that are complete applications
are executed for real through the same harness the examples job uses. A snippet
that imports, composes a frame and draws is the only proof that the documented
API still works; compiling it only proves the parentheses balance.
"""

import re
import subprocess
import sys
import textwrap
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# RST literal blocks and fenced blocks both appear in the docs.
#
# The language is checked rather than assumed. README.rst also has `.. code::
# console` blocks holding shell commands, and feeding those to compile() would
# report a dozen syntax errors that are not documentation bugs at all.
PYTHON_LANGUAGES = {"python", "py", "python3", "default", ""}

RST_BLOCK = re.compile(
    r"^\s*\.\.\s+(?:code|code-block)::\s*(\S*)\s*\n\n((?:(?:\s{3,}.*)?\n)+)", re.M
)
FENCED_BLOCK = re.compile(r"```(\w*)[ \t]*\n(.*?)```", re.S)

# Docs that show pyegui code. TODO.md is excluded: it is a plan, not reference
# documentation, and snippets in it are expected to be aspirational.
DOCS = ["README.rst", "docs/index.rst", "docs/development.rst"]


def python_snippets(path):
    """Yield (label, source) for each Python snippet in `path`."""
    text = path.read_text(encoding="utf-8")

    for language, block in RST_BLOCK.findall(text):
        if language.lower() not in PYTHON_LANGUAGES:
            continue
        lines = [
            line[3:] if line.startswith("   ") else line
            for line in block.rstrip("\n").split("\n")
        ]
        yield textwrap.dedent("\n".join(lines))

    for language, block in FENCED_BLOCK.findall(text):
        if language and language.lower() not in PYTHON_LANGUAGES:
            continue
        yield textwrap.dedent(block)


def compose_frame(source):
    """Whether the snippet draws its UI inside a container.

    egui renders nothing until the frame asks for a container, so a snippet
    that calls `run_native` without `central_panel` is not a working example
    however well it compiles.
    """
    return "central_panel" in source


def is_runnable(source):
    """True if the snippet is a complete app we can actually execute."""
    return "run_native" in source


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--list":
        for doc in DOCS:
            path = REPO_ROOT / doc
            if not path.is_file():
                continue
            for index, source in enumerate(python_snippets(path)):
                marker = "runnable" if is_runnable(source) else "syntax-only"
                print(f"{doc}#{index} {marker}")
        return 0

    total = 0
    executed = 0
    failures = []

    for doc in DOCS:
        path = REPO_ROOT / doc
        if not path.is_file():
            continue

        for index, source in enumerate(python_snippets(path)):
            total += 1
            label = f"{doc}#{index}"

            try:
                compile(source, label, "exec")
            except SyntaxError as exc:
                failures.append(f"{label}: does not parse: {exc}")
                continue

            if not is_runnable(source):
                continue

            # A complete app that composes no frame draws an empty window.
            # That is the exact bug the two stale README snippets had, so
            # failing here is the whole point of this check -- do not let it
            # fall through as "nothing to run".
            if not compose_frame(source):
                failures.append(
                    f"{label}: calls run_native but never draws a container, "
                    "so it renders an empty window"
                )
                continue

            # Execute it for real. Anything the snippet imports or calls has to
            # exist in the installed extension, so a renamed or removed binding
            # fails here rather than misleading a reader.
            proc = subprocess.run(
                [sys.executable, str(REPO_ROOT / "tests" / "smoke.py"), "--run-source"],
                input=source,
                capture_output=True,
                text=True,
                timeout=120,
            )
            executed += 1
            if proc.returncode != 0:
                detail = (proc.stderr or proc.stdout).strip().splitlines()
                tail = detail[-1] if detail else "no output"
                failures.append(f"{label}: does not run: {tail}")

    for failure in failures:
        print(f"FAIL {failure}", file=sys.stderr)

    print(f"{total - len(failures)}/{total} doc snippets OK ({executed} executed)")

    if failures:
        print(f"::error::{len(failures)} documentation snippet(s) are wrong", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())