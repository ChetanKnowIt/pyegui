"""Convert README.rst to Markdown, in place.

The rename to README.md is what makes GitHub render the README at all: GitHub
wraps a .rst file in `<div class="plain"><pre>` and applies no markup
processing, so neither RST directives nor Markdown nor raw HTML produce an image.
Verified with `gh api -H 'Accept: application/vnd.github.html'` on the .rst,
which returned zero <img> tags for both the RST figure and the Markdown form.

This is deliberately a converter rather than a hand rewrite, so the prose is
not retyped and typos cannot creep in. Every construct is counted before and
after so a silent drop is visible.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "README.rst"
DST = ROOT / "README.md"

# RST heading underline -> Markdown hash level. The document uses exactly four
# underline characters, and RST infers hierarchy from first use.
LEVELS = {"=": 1, "-": 2, "~": 3, "^": 4}


def convert(text: str) -> str:
    lines = text.split("\n")
    out = []
    fence = None  # tracks an open ``` block so headings inside are left alone
    underline_of = {}

    # First pass: record which underline char maps to which level, in order of
    # first appearance, so the mapping follows the document's own hierarchy.
    order = []
    for i, line in enumerate(lines[:-1]):
        nxt = lines[i + 1]
        if (
            re.fullmatch(r"([=\-~^\"'`#*+_:.]){2,}", nxt)
            and line.strip()
            and not re.fullmatch(r"[=\-~^\"'`#*+_:.]{2,}", line.strip())
        ):
            ch = nxt[0]
            if ch not in order:
                order.append(ch)

    # RST grid tables -> Markdown pipe tables. `+---+---+` borders with `|` cells
    # become `| a | b |`.
    #
    # Done as a whole-line rewrite BEFORE the heading pass, and then `lines` is
    # rebuilt from the result. Ordering alone is not enough: the heading loop
    # reads `lines` and reassembles `text` from it, so a table converted in
    # `text` while `lines` still held the original was silently undone -- the
    # border row then read as a heading underline and every table row became a
    # `####` heading.
    text = re.sub(
        # A grid table is a border, then alternating `|` rows and blank lines,
        # then a border. Matching only borders stopped at each one, because
        # the `|` row that follows is neither a border nor blank -- so the whole
        # table has to be described explicitly rather than as a run of borders.
        r"^[ ]*\+[-=+]{2,}\+[^\n]*\n"
        r"(?:[ ]*\n|[ ]*\|[^\n]*\n)*"
        r"^[ ]*\+[-=+]{2,}\+[^\n]*\n?",
        _grid_table,
        text,
        flags=re.M,
    )

    lines = text.split("\n")

    i = 0
    while i < len(lines):
        line = lines[i]

        # Inside a fenced code block: copy verbatim.
        if fence is not None:
            out.append(line)
            if line.strip().startswith("```"):
                fence = None
            i += 1
            continue

        # Opening a code fence.
        if line.strip().startswith("```"):
            fence = "open"
            out.append(line)
            i += 1
            continue

        # A heading: text on one line, underline on the next.
        if (
            i + 1 < len(lines)
            and re.fullmatch(r"[=\-~^\"'`#*+_]{2,}", lines[i + 1].strip())
            and line.strip()
            and not re.fullmatch(r"[=\-~^\"'`#*+_]{2,}", line.strip())
        ):
            ch = lines[i + 1].strip()[0]
            level = order.index(ch) + 1 if ch in order else 1
            out.append("#" * level + " " + line.strip())
            i += 2
            continue

        out.append(line)
        i += 1

    text = "\n".join(out)

    # `.. code:: lang` and `.. code-block:: lang` are both used in this
    # document. The first regex only handled code-block, so twelve examples
    # came through as literal directive text.
    text = re.sub(
        r"^[ \t]*\.\. (?:code|code-block)::[ ]*(\w*)\n((?:[ ].*\n|\n)*)",
        lambda m: "```"
        + m.group(1)
        + "\n"
        + _dedent(m.group(2))
        + "\n```\n",
        text,
        flags=re.M,
    )

    # `.. note::` and friends -> a blockquote, which is the Markdown idiom.
    def admonition(m):
        name, body = m.group(1), _dedent(m.group(2))
        title = {"note": "Note", "warning": "Warning"}.get(name, name.title())
        return f"> **{title}**\n>\n" + "\n".join(
            ("> " + ln if ln.strip() else ">") for ln in body.rstrip().split("\n")
        ) + "\n"

    text = re.sub(
        r"^\.\. (note|warning|attention)::\s*\n((?:[ ].*\n|\n)*)",
        admonition,
        text,
        flags=re.M,
    )

    # `.. include::` is not used in the README; fail loudly if that changes.
    if ".. include::" in text:
        raise SystemExit("README.rst contains an .. include:: directive")

    # Inline literal ``x`` -> `x`.
    #
    # This has to happen with the fenced blocks pulled out of the way first. A
    # regex for ``x`` matches inside a fence delimiter -- in "```python" the
    # leading `` is a literal opener -- which silently swallowed every fence and
    # merged the code examples into one run of backticks.
    fenced = []

    def stash(m):
        fenced.append(m.group(0))
        return f"\x00FENCE{len(fenced) - 1}\x00"

    text = re.sub(r"```[\s\S]*?```", stash, text)

    text = re.sub(r"``([^`]+)``", r"`\1`", text)

    text = re.sub(
        r"\x00FENCE(\d+)\x00",
        lambda m: fenced[int(m.group(1))],
        text,
    )

    # RST external links: `text <url>`__  ->  [text](url)
    #
    # Done after the inline-literal rewrite, so the `` ` `` around the link is
    # already a single backtick. Seven of these are in the document and each one
    # left unconverted renders as literal `text <url>`__ with a trailing __.
    text = re.sub(
        r"`([^`<]+?)\s*<((?:https?|mailto|ftp):[^>]+)>`__",
        r"[\1](\2)",
        text,
    )

    # Bare `url <target>`__ with no text, i.e. `<https://example.com>`__.
    text = re.sub(r"`<((?:https?|mailto|ftp):[^>]+)>`__", r"<\1>", text)

    # A heading needs a blank line before it in Markdown. RST does not, so a
    # heading that followed a paragraph directly came out as body text.
    text = re.sub(r"(?<!\n)\n(#{1,6} )", r"\n\n\1", text)

    # Bare URLs in angle brackets or standalone -> autolink-safe markdown.
    text = re.sub(r"<((?:https?|mailto):[^>]+)>", r"<\1>", text)

    return text


def _grid_table(m) -> str:
    """Convert one RST grid table (borders and `|` rows) to a Markdown table."""
    rows = []
    for line in m.group(0).split("\n"):
        line = line.strip()
        if not line or set(line) <= set("+-="):
            continue
        rows.append([c.strip() for c in line.strip("|").split("|")])

    if not rows:
        return m.group(0)

    width = max(len(r) for r in rows)
    rows = [r + [""] * (width - len(r)) for r in rows]

    out = ["| " + " | ".join(rows[0]) + " |"]
    out.append("| " + " | ".join(["---"] * width) + " |")
    for r in rows[1:]:
        out.append("| " + " | ".join(r) + " |")
    return "\n".join(out) + "\n"


def _literals_outside_fences(text: str) -> int:
    """Count ``x`` spans that are not inside a fenced code block."""
    count = 0
    inside = False
    for line in text.split("\n"):
        if line.strip().startswith("```"):
            inside = not inside
            continue
        if inside:
            continue
        count += len(re.findall(r"``[^`]+``", line))
    return count


def _dedent(body: str) -> str:
    lines = body.split("\n")
    while lines and not lines[0].strip():
        lines.pop(0)
    while lines and not lines[-1].strip():
        lines.pop()
    if not lines:
        return ""
    indent = min((len(ln) - len(ln.lstrip()) for ln in lines if ln.strip()), default=0)
    return "\n".join(ln[indent:] if ln.strip() else "" for ln in lines)


def main() -> int:
    src = SRC.read_text(encoding="utf-8")

    before = {
        "rst headings": len(re.findall(r"\n[=\-~^]{2,}\n", src)),
        "figure directives": src.count(".. figure::"),
        "markdown images": src.count("!["),
        "code-block": src.count(".. code-block::"),
        "admonitions": len(re.findall(r"\.\. (?:note|warning)::", src)),
        "inline literals": len(re.findall(r"``[^`]+``", src)),
        "code fences": src.count("```"),
        "code directives": len(re.findall(r"^\.\. (?:code|code-block)::", src, re.M)),
        "grid tables": len(re.findall(r"^\+[-=+]{2,}\+", src, re.M)),
        "rst links": len(re.findall(r"`[^`<]+?\s*<(?:https?|mailto|ftp):[^>]+>`__", src)),
    }

    out = convert(src)
    DST.write_text(out, encoding="utf-8")

    after = {
        "rst headings": len(re.findall(r"\n[=\-~^]{2,}\n", out)),
        "figure directives": out.count(".. figure::"),
        "markdown images": out.count("!["),
        "code-block": out.count(".. code-block::"),
        "admonitions": len(re.findall(r"\.\. (?:note|warning)::", out)),
        # Counted outside fenced blocks only. Inside them a ` `` ` sequence is
        # not an RST literal -- it is ordinary source, e.g. an f-string in the
        # Python examples -- and counting those reported three false failures
        # for a conversion that was correct.
        "inline literals": _literals_outside_fences(out),
        "code fences": out.count("```"),
        "code directives": len(re.findall(r"^\.\. (?:code|code-block)::", out, re.M)),
        "grid tables": len(re.findall(r"^\+[-=+]{2,}\+", out, re.M)),
        "rst links": len(re.findall(r"`[^`<]+?\s*<(?:https?|mailto|ftp):[^>]+>`__", out)),
    }

    print(f"{'construct':22} {'before':>7} {'after':>7}")
    for k in before:
        print(f"{k:22} {before[k]:>7} {after[k]:>7}")

    problems = []
    if after["rst headings"]:
        problems.append("RST heading underlines survived")
    # Only a directive at the start of a line is a real one. The README's prose
    # mentions the syntax by name inside `code spans`, which is not a leftover.
    if any(line.startswith(".. figure::") for line in out.split("\n")):
        problems.append("figure directives survived")
    if after["code directives"]:
        problems.append("code directives survived")
    if after["grid tables"]:
        problems.append("RST grid tables survived")
    # `text <url>`__ is RST link syntax and renders as literal text in Markdown.
    if after["rst links"]:
        problems.append(f"RST links survived ({after['rst links']})")
    # A row of a converted table starts with "| pyegui" and would otherwise be
    # reported as a heading, which is what happened before the table pass.
    if re.search(r"^#### \|", out, re.M):
        problems.append("table rows were converted to headings")
    if after["admonitions"]:
        problems.append("admonitions survived")
    if after["inline literals"]:
        problems.append("double-backtick literals survived")
    # A fence count that is not even means an unclosed block, and an odd one
    # means the fences were eaten. This is the check that catches the
    # inline-literal regex eating "```python" delimiters.
    if after["code fences"] % 2:
        problems.append(f"odd number of code fences ({after['code fences']})")
    if after["code fences"] < before["code fences"]:
        problems.append(
            f"code fences lost ({before['code fences']} -> {after['code fences']})"
        )
    if after["markdown images"] != before["markdown images"]:
        problems.append("image count changed")

    if problems:
        print("\nFAIL: " + "; ".join(problems))
        return 1

    print(f"\nwrote {DST.name}: {len(out.splitlines())} lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
