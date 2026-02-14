#!/usr/bin/env python3
"""Extract documents from CC-100 Japanese plain text (ja.txt.xz).

CC-100 format: one sentence per line, blank lines separate documents.
This script converts it to the ``<doc>`` format used by wikiextractor
so the Akaza pipeline (``akaza-data tokenize --reader=jawiki``) can
consume it without changes.

Usage:
    python3 scripts/extract-cc100.py [--limit N] [--no-filter] INPUT.txt.xz OUTPUT_DIR

Output directory structure mirrors wikiextractor:
    OUTPUT_DIR/AA/wiki_00
    OUTPUT_DIR/AA/wiki_01
    ...
    OUTPUT_DIR/AB/wiki_00
    ...
"""

import argparse
import lzma
import os
import re
import sys
import unicodedata
from collections import Counter

# Maximum number of documents per output file
ARTICLES_PER_FILE = 1000

# Translation table to remove unwanted characters:
# - ASCII control characters (0x00-0x1F) except TAB, LF, CR
# - Private Use Area (U+E000-F8FF): icon font codepoints (Font Awesome etc.)
# - Specials (U+FFF0-FFFF): replacement chars, noncharacters
# - Unicode directional/formatting control characters:
#   U+200B-200F (ZWSP, ZWNJ, ZWJ, LRM, RLM)
#   U+202A-202E (LRE, RLE, PDF, LRO, RLO)
#   U+2060-2069 (Word Joiner, invisible formatting)
#   U+FEFF (BOM / ZWNBSP)
_CONTROL_CHAR_TABLE = str.maketrans("", "", "".join(
    [chr(c) for c in range(0x20) if c not in (0x09, 0x0A, 0x0D)]
    + [chr(c) for c in range(0xE000, 0xF900)]
    + [chr(c) for c in range(0xFFF0, 0x10000)]
    + [chr(c) for c in range(0x200B, 0x2010)]
    + [chr(c) for c in range(0x202A, 0x202F)]
    + [chr(c) for c in range(0x2060, 0x206A)]
    + ["\uFEFF"]
))

# Blog boilerplate line patterns to remove
_BLOG_BOILERPLATE_RE = re.compile(r'^\[続き|^\.{2,}続き')

# --- Filters ---

MIN_DOC_LENGTH = 200  # characters


def _hiragana_ratio(text: str) -> float:
    """Return the fraction of characters that are hiragana."""
    if not text:
        return 0.0
    hiragana = sum(1 for ch in text if '\u3040' <= ch <= '\u309f')
    return hiragana / len(text)


def _line_repetition_ratio(lines: list[str]) -> float:
    """Return the fraction of lines that are duplicates."""
    if len(lines) <= 1:
        return 0.0
    counts = Counter(lines)
    repeated = sum(c - 1 for c in counts.values() if c > 1)
    return repeated / len(lines)


def _subdir_names():
    """Subdirectory names: AA, AB, AC, ..., ZZ (676 dirs)."""
    for a in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
        for b in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
            yield a + b


def main():
    parser = argparse.ArgumentParser(
        description="Convert CC-100 ja.txt.xz to <doc> format"
    )
    parser.add_argument("input", help="Input file (ja.txt.xz)")
    parser.add_argument("output_dir", help="Output directory")
    parser.add_argument(
        "--limit",
        type=int,
        default=0,
        help="Max number of documents to extract (0 = unlimited)",
    )
    parser.add_argument(
        "--no-filter",
        action="store_true",
        help="Disable all quality filters",
    )
    args = parser.parse_args()

    input_path = args.input
    output_dir = args.output_dir
    limit = args.limit
    apply_filters = not args.no_filter

    # Filter statistics
    stats = {
        "total_docs": 0,
        "filtered_short": 0,
        "filtered_hiragana": 0,
        "filtered_repetition": 0,
        "accepted": 0,
    }

    subdir_iter = _subdir_names()
    current_subdir = next(subdir_iter)
    file_index = 0
    out_file = None
    articles_in_current_file = 0
    total_articles = 0

    def open_next_file():
        nonlocal current_subdir, file_index, out_file, articles_in_current_file
        if out_file is not None:
            out_file.close()
        if total_articles > 0 and total_articles % ARTICLES_PER_FILE == 0:
            file_index += 1
            if file_index >= 100:
                file_index = 0
                current_subdir = next(subdir_iter)
        dir_path = os.path.join(output_dir, current_subdir)
        os.makedirs(dir_path, exist_ok=True)
        file_path = os.path.join(dir_path, f"wiki_{file_index:02d}")
        out_file = open(file_path, "a", encoding="utf-8")
        articles_in_current_file = 0

    def flush_doc(lines, doc_id):
        nonlocal total_articles, articles_in_current_file, out_file

        if not lines:
            return False

        stats["total_docs"] += 1
        text = "\n".join(lines)

        if apply_filters:
            # Filter 1: minimum document length
            if len(text) < MIN_DOC_LENGTH:
                stats["filtered_short"] += 1
                return False

            # Filter 2: hiragana ratio
            if _hiragana_ratio(text) < 0.10:
                stats["filtered_hiragana"] += 1
                return False

            # Filter 3: line repetition
            if _line_repetition_ratio(lines) >= 0.30:
                stats["filtered_repetition"] += 1
                return False

        stats["accepted"] += 1
        out_file.write(f'<doc id="{doc_id}" url="" title="cc100_{doc_id}">\n')
        out_file.write(text)
        out_file.write("\n</doc>\n")
        articles_in_current_file += 1
        total_articles += 1
        if articles_in_current_file >= ARTICLES_PER_FILE:
            open_next_file()
        return True

    open_next_file()

    doc_lines = []
    doc_id = 0

    with lzma.open(input_path, "rt", encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if line == "":
                # Document boundary
                if doc_lines:
                    if flush_doc(doc_lines, doc_id):
                        doc_id += 1
                    doc_lines = []
                    if limit > 0 and total_articles >= limit:
                        break
            else:
                # 1. Strip control/formatting characters
                line = line.translate(_CONTROL_CHAR_TABLE)
                # 2. NFKC normalization (CJK compat → unified, halfwidth → fullwidth kana, etc.)
                line = unicodedata.normalize('NFKC', line)
                # 3. Skip empty lines and blog boilerplate
                if line and not _BLOG_BOILERPLATE_RE.search(line):
                    doc_lines.append(line)

    # Flush last document if file doesn't end with blank line
    if doc_lines and (limit == 0 or total_articles < limit):
        flush_doc(doc_lines, doc_id)

    if out_file is not None:
        out_file.close()

    print(f"Extracted {total_articles} documents to {output_dir}", file=sys.stderr)
    if apply_filters:
        print(f"Filter statistics:", file=sys.stderr)
        print(f"  Total documents seen:    {stats['total_docs']}", file=sys.stderr)
        print(f"  Filtered (short <{MIN_DOC_LENGTH}):  {stats['filtered_short']}", file=sys.stderr)
        print(f"  Filtered (hiragana <10%): {stats['filtered_hiragana']}", file=sys.stderr)
        print(f"  Filtered (line repeat):   {stats['filtered_repetition']}", file=sys.stderr)
        print(f"  Accepted:                 {stats['accepted']}", file=sys.stderr)
        if stats['total_docs'] > 0:
            pct = stats['accepted'] / stats['total_docs'] * 100
            print(f"  Acceptance rate:          {pct:.1f}%", file=sys.stderr)


if __name__ == "__main__":
    main()
