#!/usr/bin/env bash
#
# format-tables.sh — Format markdown tables with aligned columns.
#
# Usage: ./format-tables.sh <file.md>
#
# Rules applied:
#   - Space between | and neighbouring -  in separator rows
#   - At least one space before and after each cell value
#   - Each column is as wide as its longest cell value

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <file.md>" >&2
    exit 1
fi

file="$1"

if [[ ! -f "$file" ]]; then
    echo "Error: file '$file' not found." >&2
    exit 1
fi

python3 - "$file" <<'PYTHON'
import sys
import unicodedata
import re


def display_width(s):
    """Calculate the display width of a string, accounting for wide characters."""
    w = 0
    for ch in s:
        eaw = unicodedata.east_asian_width(ch)
        if eaw in ("F", "W"):
            w += 2
        else:
            w += 1
    return w


def pad_to_width(s, target_width):
    """Pad string with spaces to reach target display width."""
    return s + " " * (target_width - display_width(s))


def is_separator(row):
    """Check if a row is a separator (only contains -, :, and whitespace)."""
    return all(re.fullmatch(r"[-:\s]*", cell) for cell in row)


def parse_row(line):
    """Parse a markdown table row into a list of trimmed cell values."""
    line = line.strip()
    if line.startswith("|"):
        line = line[1:]
    if line.endswith("|"):
        line = line[:-1]
    return [cell.strip() for cell in line.split("|")]


def format_table(rows):
    """Format a list of parsed rows into aligned markdown table lines."""
    # Find which rows are separators
    sep_flags = [is_separator(row) for row in rows]

    # Determine number of columns
    ncols = max(len(row) for row in rows)

    # Pad rows with fewer columns
    for row in rows:
        while len(row) < ncols:
            row.append("")

    # Calculate max display width per column (ignoring separator rows)
    col_widths = [0] * ncols
    for i, row in enumerate(rows):
        if sep_flags[i]:
            continue
        for j, cell in enumerate(row):
            w = display_width(cell)
            if w > col_widths[j]:
                col_widths[j] = w

    # Ensure minimum width of 1
    col_widths = [max(w, 1) for w in col_widths]

    # Build output lines
    lines = []
    for i, row in enumerate(rows):
        if sep_flags[i]:
            cells = [" " + "-" * col_widths[j] + " " for j in range(ncols)]
        else:
            cells = [
                " " + pad_to_width(row[j], col_widths[j]) + " "
                for j in range(ncols)
            ]
        lines.append("|" + "|".join(cells) + "|")
    return lines


def main():
    filepath = sys.argv[1]
    with open(filepath, "r", encoding="utf-8") as f:
        input_lines = f.readlines()

    output = []
    table_rows = []
    in_table = False

    for line in input_lines:
        stripped = line.rstrip("\n")
        if re.match(r"^[ \t]*\|", stripped):
            if not in_table:
                in_table = True
            table_rows.append(parse_row(stripped))
        else:
            if in_table:
                output.extend(format_table(table_rows))
                table_rows = []
                in_table = False
            output.append(stripped)

    if in_table:
        output.extend(format_table(table_rows))

    result = "\n".join(output) + "\n"
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(result)


main()
PYTHON
