"""Check that the documentation describes the module that actually ships.

The README and CHANGELOG state a name count and a feature list. Those were
written from memory and were wrong: the CHANGELOG advertised `Hsva`, `Hsl`,
`Oklch` and `color_edit_oklch`, none of which have ever existed, and both files
claimed 121 names when the module exports 123. Nothing caught it, because
`check.yml`'s export gate compares `expected_exports.py` against the built
module -- both of which are correct -- and never looks at the prose.

This closes that gap. It derives the numbers from `tests/expected_exports.py`
and fails when the documentation disagrees, so a claim cannot quietly become
false when a batch lands.

It also checks that every function the CHANGELOG presents as shipped under
"Added" really is exported, which is the specific claim that went wrong.
"""

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "tests"))

import expected_exports as manifest  # noqa: E402

DOCS = {
    "README.rst": REPO_ROOT / "README.rst",
    "CHANGELOG.md": REPO_ROOT / "CHANGELOG.md",
}

# Prose that legitimately names something that is not a Python export: the
# egui types behind a binding, Rust crates, and workflow names.
ALLOWED_NON_EXPORTS = {
    # egui / Rust type names behind the bindings
    "Window", "CentralPanel", "SidePanel", "TopBottomPanel", "Area", "Popup",
    "Modal", "Resize", "Scene", "MenuBar", "CollapsingHeader", "Frame",
    "Ui", "Button", "Checkbox", "ComboBox", "Slider", "Grid", "Image",
    "RichText", "Response", "Painter", "RetainedImage", "Table",
    "TableBuilder", "ScrollArea", "TextEdit", "DragValue", "RadioButton",
    "ProgressBar", "NaiveDate", "DatePickerButton", "BoxedWidget",
    "PopupAnchor", "PopupCloseBehavior", "Sizing", "StripBuilder",
    "UiBuilder", "PopupsAnchor", "NativeOptions",
    # crates and tooling
    "egui", "eframe", "egui_extras", "maturin", "cargo", "rustup", "uv",
    "docker", "Makefile", "make", "pip", "GitHub", "Actions",
    # workflows and files
    "check", "examples", "release", "screenshot", "CI", "Roadmap",
    "Docker", "CHANGELOG", "TODO", "README",
}


def counts():
    """(total, classes, functions, response_variants) as CI counts them."""
    classes = len(manifest.CLASSES)
    functions = len(manifest.FUNCTIONS)
    response = len(manifest.RESPONSE_VARIANTS)
    # CI reports functions as plain + response variants.
    return classes + functions + response, classes, functions + response, response


def main():
    total, classes, functions, response = counts()
    failures = []

    readme = DOCS["README.rst"].read_text(encoding="utf-8")
    changelog = DOCS["CHANGELOG.md"].read_text(encoding="utf-8")

    # 1. The name count, stated in the README.
    stated = re.search(r"\*\*(\d+) names\*\*", readme)
    if not stated:
        failures.append("README.rst does not state a name count in the form '**N names**'")
    elif int(stated.group(1)) != total:
        failures.append(
            f"README.rst says {stated.group(1)} names; the module exports {total} "
            f"({classes} classes, {functions} functions, {response} response variants)"
        )

    # 2. The same count in the CHANGELOG, if it states one.
    for m in re.finditer(r"\*\*(\d+) names\*\*|(\d+) names", changelog):
        n = int(m.group(1) or m.group(2))
        if n != total:
            failures.append(f"CHANGELOG.md says {n} names; the module exports {total}")

    # 3. Every snake_case name the CHANGELOG presents as a shipped function
    #    under "Added" must actually exist.
    exported = set(manifest.CLASSES) | set(manifest.FUNCTIONS) | set(
        manifest.RESPONSE_VARIANTS
    )
    ctx_methods = set(
        re.findall(r"fn (\w+)\(&self\)", (REPO_ROOT / "src" / "lib.rs").read_text(encoding="utf-8"))
    )

    # Only names presented as shipped count. A line like "Side, top and bottom
    # panels, `modal`, `popup` and `popup_menu` are not in this release" names
    # three things precisely to say they are absent, and flagging those would
    # punish the changelog for being accurate.
    NOT_SHIPPED = re.compile(
        r"(not (?:in|yet|shipped|available|adopted)[^.]*\.|are not|cannot|"
        r"not implemented|yet\.|TODO)",
        re.I,
    )

    for line in changelog.split("\n"):
        stripped = line.strip()
        if not stripped.startswith(("-", "*")):
            continue
        # Skip the bullet if it says these are absent, or if it is documenting
        # a plan rather than a shipped name.
        if NOT_SHIPPED.search(stripped):
            continue

        for name in sorted(set(re.findall(r"`([a-z][a-z0-9_]{3,})`", stripped))):
            if name in exported or name in ctx_methods or name in ALLOWED_NON_EXPORTS:
                continue
            failures.append(
                f"CHANGELOG.md presents `{name}` as shipped, but it is not exported"
            )

    # Class names are CamelCase and matter just as much: the old changelog
    # advertised Hsva, Hsl and Oklch, which never existed.
    for line in changelog.split("\n"):
        stripped = line.strip()
        if not stripped.startswith(("-", "*")):
            continue
        if NOT_SHIPPED.search(stripped):
            continue
        for name in sorted(set(re.findall(r"`([A-Z][A-Za-z0-9]{2,})`", stripped))):
            if name in exported or name in ALLOWED_NON_EXPORTS:
                continue
            # Python builtins and prose, not pyegui exports.
            if name in {"Rust", "Linux", "Windows", "MacOS", "GitHub", "Actions"}:
                continue
            if name in {"ValueError", "TypeError", "RuntimeError", "OSError"}:
                continue
            failures.append(
                f"CHANGELOG.md presents class `{name}` as shipped, but it is not exported"
            )

    for failure in failures:
        print(f"FAIL {failure}", file=sys.stderr)

    print(
        f"module exports {total} names ({classes} classes, "
        f"{functions} functions, {response} response variants)"
    )
    print(f"{len(failures)} documentation claim(s) disagree with the module")

    if failures:
        print(
            "::error::the docs describe an API that does not match "
            "tests/expected_exports.py",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())