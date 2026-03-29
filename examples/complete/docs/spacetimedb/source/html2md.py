#!/usr/bin/env -S python3 -u
"""Convert Docusaurus HTML docs to Markdown.

Requires: pip install beautifulsoup4
"""

import re
import sys
from pathlib import Path

try:
    from bs4 import BeautifulSoup, NavigableString, Tag
except ImportError:
    print("beautifulsoup4 is required. Install with:")
    print("  pip install beautifulsoup4")
    sys.exit(1)


def extract_main_content(soup):
    """Extract the main documentation content div."""
    content = soup.find("div", class_="theme-doc-markdown")
    if content:
        return content
    # Fallback: try the article tag
    article = soup.find("article")
    if article:
        for el in article.find_all(
            class_=re.compile(r"breadcrumb|version-badge|tocCollapsible|tocMobile")
        ):
            el.decompose()
        return article
    return None


def extract_code_text(code_el):
    """Extract plain text from a syntax-highlighted <code> element.

    Docusaurus renders each line as <span class="line">...<br></span> or
    <span class="token-line">...<br></span>.  We iterate over line spans,
    collect the text of each, and join with newlines.
    """
    line_spans = code_el.find_all(
        "span",
        class_=lambda c: c and any(x in c for x in ("token-line", "line")),
        recursive=False,
    )
    if line_spans:
        lines = []
        for span in line_spans:
            # get_text() already decodes HTML entities
            text = span.get_text()
            # Remove trailing newline that <br> produces
            text = text.rstrip("\n")
            lines.append(text)
        return "\n".join(lines)
    # Fallback: just use the whole text
    return code_el.get_text()


def detect_language(element):
    """Walk up the tree to find a language-xxx class."""
    el = element
    while el:
        if isinstance(el, Tag):
            for cls in el.get("class", []):
                if cls.startswith("language-"):
                    lang = cls[len("language-") :]
                    if lang in ("text", "plain"):
                        return ""
                    return lang
        el = el.parent
    return ""


# ---------------------------------------------------------------------------
# Recursive HTML -> Markdown converter
# ---------------------------------------------------------------------------


def convert(node):
    """Convert an HTML node (and its subtree) to Markdown text."""
    if isinstance(node, NavigableString):
        return str(node)

    if not isinstance(node, Tag):
        return ""

    tag = node.name
    classes = " ".join(node.get("class", []))

    # ---- skip elements we never want ----
    if node.get("hidden") is not None and "tabItem" not in classes:
        return ""
    if "hash-link" in classes:
        return ""
    if tag == "button":
        return ""
    if tag in ("svg", "nav", "aside", "script", "style", "link"):
        return ""
    if "tocCollapsible" in classes or "tocMobile" in classes:
        return ""
    if "theme-doc-version-badge" in classes:
        return ""
    if "theme-doc-footer" in classes or "pagination-nav" in classes:
        return ""

    # ---- headings ----
    if tag in ("h1", "h2", "h3", "h4", "h5", "h6"):
        level = int(tag[1])
        text = node.get_text().replace("\u200b", "").strip()
        return f"\n\n{'#' * level} {text}\n\n"

    # ---- paragraph ----
    if tag == "p":
        inner = _children(node).strip()
        return f"\n\n{inner}\n\n"

    # ---- links ----
    if tag == "a":
        href = node.get("href", "")
        text = _children(node).strip()
        if not text:
            return ""
        if href:
            return f"[{text}]({href})"
        return text

    # ---- inline formatting ----
    if tag == "strong" or tag == "b":
        return f"**{_children(node).strip()}**"
    if tag == "em" or tag == "i":
        return f"*{_children(node).strip()}*"

    # ---- inline code (not inside <pre>) ----
    if tag == "code" and not node.find_parent("pre"):
        return f"`{node.get_text()}`"

    # ---- code block containers (language-xxx wrapper div) ----
    if tag == "div" and re.search(r"language-|codeBlockContainer", classes):
        pre = node.find("pre")
        if pre:
            code_el = pre.find("code")
            code_text = extract_code_text(code_el) if code_el else pre.get_text()
            lang = detect_language(node)
            return f"\n\n```{lang}\n{code_text}\n```\n\n"

    # ---- standalone pre/code ----
    if tag == "pre":
        code_el = node.find("code")
        code_text = extract_code_text(code_el) if code_el else node.get_text()
        lang = detect_language(node)
        return f"\n\n```{lang}\n{code_text}\n```\n\n"

    # ---- tab containers ----
    if tag == "div" and re.search(r"tabs-container|tabList_", classes):
        return _convert_tabs(node)

    # ---- tab panels (standalone – usually handled inside _convert_tabs) ----
    if tag == "div" and "tabItem" in classes:
        if node.get("hidden") is not None:
            return ""
        return _children(node)

    # ---- admonitions ----
    if tag == "div" and re.search(r"theme-admonition|admonition_", classes):
        return _convert_admonition(node)

    # ---- blockquotes ----
    if tag == "blockquote":
        inner = _children(node).strip()
        quoted = "\n".join(f"> {line}" for line in inner.split("\n"))
        return f"\n\n{quoted}\n\n"

    # ---- unordered lists ----
    if tag == "ul":
        items = []
        for li in node.find_all("li", recursive=False):
            item = _children(li).strip()
            # Indent nested lines
            lines = item.split("\n")
            first = f"- {lines[0]}"
            rest = "\n".join(f"  {l}" for l in lines[1:]) if len(lines) > 1 else ""
            items.append(first + ("\n" + rest if rest else ""))
        return "\n\n" + "\n".join(items) + "\n\n"

    # ---- ordered lists ----
    if tag == "ol":
        items = []
        start = int(node.get("start", 1))
        for i, li in enumerate(node.find_all("li", recursive=False), start):
            item = _children(li).strip()
            lines = item.split("\n")
            first = f"{i}. {lines[0]}"
            rest = "\n".join(f"   {l}" for l in lines[1:]) if len(lines) > 1 else ""
            items.append(first + ("\n" + rest if rest else ""))
        return "\n\n" + "\n".join(items) + "\n\n"

    # ---- list items (when reached directly) ----
    if tag == "li":
        return _children(node)

    # ---- horizontal rule ----
    if tag == "hr":
        return "\n\n---\n\n"

    # ---- images ----
    if tag == "img":
        alt = node.get("alt", "")
        src = node.get("src", "")
        return f"![{alt}]({src})"

    # ---- HTML tables ----
    if tag == "table":
        return _convert_table(node)

    # ---- line break ----
    if tag == "br":
        return "\n"

    # ---- details / summary ----
    if tag == "details":
        summary = node.find("summary")
        summary_text = summary.get_text(strip=True) if summary else ""
        if summary:
            summary.decompose()
        inner = _children(node).strip()
        return f"\n\n<details>\n<summary>{summary_text}</summary>\n\n{inner}\n\n</details>\n\n"

    # ---- default: just recurse ----
    return _children(node)


def _children(node):
    """Convert all children and concatenate."""
    return "".join(convert(child) for child in node.children)


# ---------------------------------------------------------------------------
# Tab handling
# ---------------------------------------------------------------------------


def _convert_tabs(container):
    """Convert a Docusaurus tab container into Markdown sections."""
    # Collect tab labels
    labels = []
    tablist = container.find("ul", attrs={"role": "tablist"})
    if tablist:
        for tab in tablist.find_all("li", attrs={"role": "tab"}):
            labels.append(tab.get_text(strip=True))

    # Collect all panels (including hidden ones)
    panels = container.find_all("div", attrs={"role": "tabpanel"})

    parts = []
    for i, panel in enumerate(panels):
        label = labels[i] if i < len(labels) else f"Tab {i + 1}"
        # Temporarily unhide so convert() processes children
        was_hidden = panel.get("hidden")
        if was_hidden is not None:
            del panel["hidden"]
        inner = _children(panel).strip()
        if was_hidden is not None:
            panel["hidden"] = ""
        if inner:
            parts.append(f"\n\n#### {label}\n\n{inner}")

    return "\n".join(parts)


# ---------------------------------------------------------------------------
# Admonition handling
# ---------------------------------------------------------------------------


def _convert_admonition(element):
    """Convert a Docusaurus admonition to a Markdown blockquote callout."""
    classes = " ".join(element.get("class", []))

    # Detect type
    admonition_type = "note"
    for t in ("tip", "warning", "danger", "caution", "info", "important"):
        if t in classes.lower():
            admonition_type = t
            break

    # Extract heading text
    heading_el = element.find(class_=re.compile(r"admonitionHeading"))
    heading = heading_el.get_text(strip=True) if heading_el else admonition_type.capitalize()
    if heading_el:
        heading_el.decompose()

    # Extract body
    content_el = element.find(class_=re.compile(r"admonitionContent"))
    if content_el:
        body = _children(content_el).strip()
    else:
        body = _children(element).strip()

    lines = body.split("\n")
    quoted = "\n".join(f"> {line}" for line in lines)
    return f"\n\n> **{heading}**\n>\n{quoted}\n\n"


# ---------------------------------------------------------------------------
# Table handling
# ---------------------------------------------------------------------------


def _convert_table(table_el):
    """Convert an HTML <table> to a Markdown table."""
    rows = []

    thead = table_el.find("thead")
    if thead:
        for tr in thead.find_all("tr"):
            cells = [_children(cell).strip() for cell in tr.find_all(["th", "td"])]
            rows.append(cells)

    tbody = table_el.find("tbody") or table_el
    for tr in tbody.find_all("tr", recursive=(tbody == table_el)):
        cells = [_children(cell).strip() for cell in tr.find_all(["td", "th"])]
        if cells:
            rows.append(cells)

    if not rows:
        return ""

    ncols = max(len(r) for r in rows)
    # Pad short rows
    for r in rows:
        while len(r) < ncols:
            r.append("")

    out = []
    out.append("| " + " | ".join(rows[0]) + " |")
    out.append("| " + " | ".join(["---"] * ncols) + " |")
    for row in rows[1:]:
        out.append("| " + " | ".join(row) + " |")

    return "\n\n" + "\n".join(out) + "\n\n"


# ---------------------------------------------------------------------------
# Post-processing
# ---------------------------------------------------------------------------


def clean_markdown(text):
    """Tidy up the raw Markdown output."""
    # Collapse runs of 3+ blank lines to 2
    text = re.sub(r"\n{3,}", "\n\n", text)
    # Strip trailing whitespace per line
    text = "\n".join(line.rstrip() for line in text.split("\n"))
    # Ensure single trailing newline
    text = text.strip() + "\n"
    return text


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def convert_file(html_path, md_path):
    """Read an HTML file, extract docs content, write Markdown."""
    html = html_path.read_text(encoding="utf-8")
    soup = BeautifulSoup(html, "html.parser")

    content = extract_main_content(soup)
    if content is None:
        print(f"  WARNING: no main content found in {html_path.name}")
        return False

    md = convert(content)
    md = clean_markdown(md)

    md_path.write_text(md, encoding="utf-8")
    return True


def main():
    script_dir = Path(__file__).resolve().parent
    html_files = sorted(script_dir.glob("*.html"))

    if not html_files:
        print(f"No HTML files found in {script_dir}")
        sys.exit(1)

    print(f"Converting {len(html_files)} HTML files to Markdown…\n")

    ok = 0
    for html_path in html_files:
        md_path = html_path.with_suffix(".md")
        print(f"  {html_path.name}  →  {md_path.name}")
        if convert_file(html_path, md_path):
            ok += 1

    print(f"\nDone. Converted {ok}/{len(html_files)} files.")


if __name__ == "__main__":
    main()
