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
#
# The awk program runs in the C locale and decodes UTF-8 itself, so that column
# widths do not depend on the locale awk was started in. East Asian Wide and
# Fullwidth characters count as two columns. Their ranges are the static table
# below rather than a Unicode database lookup, so a character this table does
# not list counts as one column wide.
#
# The line ending of the first line is the line ending of the whole result. It is
# detected here rather than inside awk, because some awk builds open their input
# in text mode and strip the carriage return before the program sees it.

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

program=""
formatted=""
trap '[[ -n "$program" ]] && rm -f "$program"; [[ -n "$formatted" ]] && rm -f "$formatted"' EXIT

program="$(mktemp)"
formatted="$(mktemp)"

line_ending=$'\n'
first_line=""
IFS= read -r first_line < "$file" || true
if [[ "$first_line" == *$'\r' ]]; then
    line_ending=$'\r\n'
fi

cat > "$program" <<'AWK'
function add_wide_range(low, high) {
    wide_range_count++
    wide_range_low[wide_range_count] = low
    wide_range_high[wide_range_count] = high
}

function is_wide(codepoint,   range) {
    for (range = 1; range <= wide_range_count; range++)
        if (codepoint >= wide_range_low[range] && codepoint <= wide_range_high[range])
            return 1
    return 0
}

# The display width of a UTF-8 encoded string, in terminal columns.
function display_width(text,   position, length_in_bytes, first_byte, continuation, sequence_length, codepoint, width) {
    width = 0
    position = 1
    length_in_bytes = length(text)

    while (position <= length_in_bytes) {
        first_byte = byte_value[substr(text, position, 1)]

        if (first_byte < 0xC0) {
            sequence_length = 1
            codepoint = first_byte
        } else if (first_byte < 0xE0) {
            sequence_length = 2
            codepoint = first_byte % 0x20
        } else if (first_byte < 0xF0) {
            sequence_length = 3
            codepoint = first_byte % 0x10
        } else {
            sequence_length = 4
            codepoint = first_byte % 0x08
        }

        if (position + sequence_length - 1 > length_in_bytes) {
            # Truncated sequence: count the remaining bytes as one column each.
            return width + (length_in_bytes - position + 1)
        }

        for (continuation = 1; continuation < sequence_length; continuation++)
            codepoint = codepoint * 0x40 + byte_value[substr(text, position + continuation, 1)] % 0x40

        width += is_wide(codepoint) ? 2 : 1
        position += sequence_length
    }

    return width
}

function repeat(character, count,   result) {
    result = ""
    while (length(result) < count) result = result character
    return result
}

function pad_to_width(text, target_width) {
    return text repeat(" ", target_width - display_width(text))
}

function trim(text) {
    sub(/^[[:space:]]+/, "", text)
    sub(/[[:space:]]+$/, "", text)
    return text
}

function is_separator(row,   column) {
    for (column = 1; column <= cell_count[row]; column++)
        if (cell[row, column] !~ /^[-:[:space:]]*$/) return 0
    return 1
}

function format_table(   row, column, column_count, width, separator, line) {
    if (row_count == 0) return

    column_count = 0
    for (row = 1; row <= row_count; row++)
        if (cell_count[row] > column_count) column_count = cell_count[row]

    for (row = 1; row <= row_count; row++)
        for (column = cell_count[row] + 1; column <= column_count; column++)
            cell[row, column] = ""

    for (column = 1; column <= column_count; column++) column_width[column] = 1

    for (row = 1; row <= row_count; row++) {
        if (is_separator(row)) continue
        for (column = 1; column <= column_count; column++) {
            width = display_width(cell[row, column])
            if (width > column_width[column]) column_width[column] = width
        }
    }

    for (row = 1; row <= row_count; row++) {
        separator = is_separator(row)
        line = "|"
        for (column = 1; column <= column_count; column++) {
            if (separator)
                line = line " " repeat("-", column_width[column]) " |"
            else
                line = line " " pad_to_width(cell[row, column], column_width[column]) " |"
        }
        print line
    }

    row_count = 0
    delete cell
    delete cell_count
    delete column_width
}

BEGIN {
    for (code = 0; code < 256; code++) byte_value[sprintf("%c", code)] = code

    wide_range_count = 0
    add_wide_range(0x1100, 0x115F)
    add_wide_range(0x2E80, 0x303E)
    add_wide_range(0x3041, 0x33FF)
    add_wide_range(0x3400, 0x4DBF)
    add_wide_range(0x4E00, 0x9FFF)
    add_wide_range(0xA000, 0xA4CF)
    add_wide_range(0xA960, 0xA97F)
    add_wide_range(0xAC00, 0xD7A3)
    add_wide_range(0xF900, 0xFAFF)
    add_wide_range(0xFE10, 0xFE19)
    add_wide_range(0xFE30, 0xFE6F)
    add_wide_range(0xFF00, 0xFF60)
    add_wide_range(0xFFE0, 0xFFE6)
    add_wide_range(0x1F300, 0x1F64F)
    add_wide_range(0x1F900, 0x1F9FF)
    add_wide_range(0x20000, 0x2FFFD)
    add_wide_range(0x30000, 0x3FFFD)

    row_count = 0
    ORS = output_record_separator
}

{
    line = $0
    sub(/\r$/, "", line)

    if (line ~ /^[ \t]*\|/) {
        row = trim(line)
        sub(/^\|/, "", row)
        sub(/\|$/, "", row)
        row_count++
        cell_count[row_count] = split(row, cells, "\\|")
        for (column = 1; column <= cell_count[row_count]; column++)
            cell[row_count, column] = trim(cells[column])
    } else {
        format_table()
        print line
    }
}

END {
    format_table()
}
AWK

LC_ALL=C awk -v output_record_separator="$line_ending" -f "$program" "$file" > "$formatted"

cat "$formatted" > "$file"
