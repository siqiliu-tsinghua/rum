"""Independent TSV parser/count audit, using only Python's standard library.

This is independent of the Rust report aggregation, not a second Rust resolver.
Read every reported source span and save one complete witness per reason class.
"""

import collections
import csv
from pathlib import Path
import sys


def read(directory, name):
    with (directory / name).open(newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t", quoting=csv.QUOTE_NONE))


def key(row):
    return tuple(row[k] for k in (
        "definition", "file", "start_line", "start_column", "end_line",
        "end_column", "kind", "target", "reason",
    ))


def reason_class(reason):
    method_classes = {
        "method syntax: opaque imported trait candidate ": "method syntax: opaque imported trait candidate <target>",
        "method syntax: prelude trait candidate ": "method syntax: prelude trait candidate <method> at <receiver>",
        "method syntax: earlier trait candidate ": "method syntax: earlier trait candidate <trait::method> at <receiver>",
        "method syntax: path ": "method syntax: path <path> has <n> exact bindings",
        "method syntax: opaque or ambiguous type ": "method syntax: opaque or ambiguous type <type>",
        "method syntax: trait ": "method syntax: trait <trait> has unmodeled supertraits or generated candidates",
    }
    for prefix, group in method_classes.items():
        if reason.startswith(prefix):
            return group
    if reason.startswith("path ") and reason.endswith(" exact bindings"):
        return "path <path> has <n> exact bindings"
    if reason.startswith("not a resolved function definition:"):
        return "not a resolved function definition: <target>"
    if reason.startswith("ambiguous or conditional function "):
        return "ambiguous or conditional function <target>"
    return reason


def main(workspace, directory):
    raw = read(directory, "rows.tsv")
    unique = read(directory, "deduplicated.tsv")
    grouped = collections.defaultdict(list)
    for row in raw:
        assert row["population"] == directory.name
        assert row["class"] == reason_class(row["reason"])
        assert row["witness"] and row["reason"] and row["definition"]
        grouped[key(row)].append(row)
    assert len(unique) == len(grouped)
    assert {key(r) for r in unique} == set(grouped)
    for row in unique:
        assert row["roots"].split(" | ") == sorted({r["roots"] for r in grouped[key(row)]})
        for original in grouped[key(row)]:
            assert f'{original["roots"]}: {original["witness"]}' in row["witness"].split(" | ")

    for filename, field in [("reason-table.tsv", "class"), ("exact-reasons.tsv", "reason")]:
        counts = collections.Counter((r["kind"], r[field]) for r in raw)
        dedup_counts = collections.Counter((r["kind"], r[field]) for r in unique)
        summary = read(directory, filename)
        assert [(r["kind"], r["reason"]) for r in summary] == sorted(counts, key=lambda k: (-dedup_counts[k], k))
        assert {(r["kind"], r["reason"]): int(r["raw"]) for r in summary} == counts
        assert {(r["kind"], r["reason"]): int(r["deduplicated"]) for r in summary} == dedup_counts

    visited = read(directory, "visited.tsv")
    expected = read(directory, "expected-definitions.tsv")
    assert len(visited) == len(expected)
    assert {(r["definition"], r["span"]) for r in visited} == {(r["definition"], r["span"]) for r in expected}
    roots = read(directory, "roots.tsv")
    incidences = read(directory, "root-incidence.tsv")
    assert {r["root"] for r in roots} == {r["root"] for r in incidences}
    incidence = {r["root"]: r for r in incidences}
    root_rows = collections.defaultdict(list)
    for row in raw:
        root_rows[row["roots"]].append(row)
    root_bodies = collections.Counter(name for row in visited for name in row["roots"].split(" | "))
    for root in roots:
        name = root["root"]
        rows = root_rows[name]
        bodies = root_bodies[name]
        assert rows or bodies, f"root disappeared: {name}"
        assert (len(rows), len({key(r) for r in rows}), bodies) == tuple(int(incidence[name][k]) for k in ("raw", "deduplicated", "definitions"))
        if root["reason"]:
            assert any(r["reason"] == root["reason"] and r["target"] == root["target"] for r in rows)
    first = read(directory, "first-failure.tsv")
    first_ok = (directory / "first-ok.txt").read_text().splitlines()
    assert len(first) + len(first_ok) == len(roots)
    assert {r["roots"] for r in first}.isdisjoint(first_ok)
    assert {r["roots"] for r in first} | set(first_ok) == set(incidence)

    # Every site is actually read, not merely counted. This catches nonexistent
    # files, out-of-range spans and accidental use of <root> placeholders.
    sources = {}
    samples = {}
    for index, row in enumerate(unique + first):
        file = row["file"]
        if file not in sources:
            sources[file] = (workspace / file).read_text().splitlines()
        lines = sources[file]
        sl, sc, el, ec = (int(row[k]) for k in ("start_line", "start_column", "end_line", "end_column"))
        assert 1 <= sl <= el <= len(lines)
        assert 1 <= sc <= len(lines[sl - 1]) + 1 and 1 <= ec <= len(lines[el - 1]) + 1
        assert (sl, sc) < (el, ec)
        selected = lines[sl - 1:el]
        selected[-1] = selected[-1][:ec - 1]
        selected[0] = selected[0][sc - 1:]
        snippet = "\n".join(selected)
        assert snippet.strip()
        if index < len(unique):
            k = (row["kind"], row["class"])
            candidate = (len(snippet), file, sl, sc, row, snippet)
            if k not in samples or candidate[:4] < samples[k][:4]:
                samples[k] = candidate
    with (directory / "span-samples.txt").open("w") as stream:
        for (kind, reason), (_, file, sl, sc, row, snippet) in sorted(samples.items()):
            stream.write(f"{kind}\t{reason}\n{row['definition']}\n{file}:{sl}:{sc}-{row['end_line']}:{row['end_column']}\n{row['reason']}\n{snippet}\n\n")
    print(f"INDEPENDENT_AUDIT {directory.name} roots={len(roots)} definitions={len(visited)} raw={len(raw)} deduplicated={len(unique)} reason_classes={len(samples)} all_spans_read=true")


if __name__ == "__main__":
    main(Path(sys.argv[1]), Path(sys.argv[2]))
