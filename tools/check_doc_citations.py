"""Check that every `path:line` citation in a Markdown doc points at an existing, non-blank line.

    python3 tools/check_doc_citations.py docs/champion-guide-addendum.md

Prints each citation with the line it lands on, so a reviewer can see it still says what the doc claims.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]


def check(doc):
    text = (ROOT / doc).read_text(encoding='utf-8')
    bad = checked = 0
    for m in re.finditer(r'`([^`\s][^`]*?):(\d+)`', text):
        path, line = ROOT / m.group(1), int(m.group(2))
        checked += 1
        if not path.is_file():
            print('MISSING', m.group(1))
            bad += 1
            continue
        lines = path.read_text(encoding='utf-8', errors='replace').splitlines()
        if line > len(lines) or not lines[line - 1].strip():
            print('BAD LINE', m.group(0))
            bad += 1
        else:
            print(f'{m.group(1)}:{line}: {lines[line - 1].strip()[:100]}')
    print(f'{doc}: {checked} citations, {bad} bad')
    return bad


if __name__ == '__main__':
    sys.exit(1 if sum(check(d) for d in sys.argv[1:] or ['docs/champion-guide-addendum.md']) else 0)
