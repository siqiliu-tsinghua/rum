"""All A2 method selections, observed with stable rustc's item deprecation lint.

Each target gets a unique marker. Only caller functions deny deprecation, so
upstream crates still compile and cannot hide downstream claims. Positive
errors must name that marker at the exact method token; negative mutations
touch distinct same-named inherent methods with independent control calls.
No expected receiver type, argument coercion, or call expression is changed.
"""
import csv
import json
from pathlib import Path
import shutil
import sys
import time

sys.dont_write_bytecode = True
from scratch import Scratch, offset, read_tsv
from corpus import target
from run import agreement


def diagnostics(result):
    records = []
    for line in (result.stdout + result.stderr).splitlines():
        try:
            records.append(json.loads(line))
        except (ValueError, UnicodeDecodeError):
            pass
    return [r['message'] if r.get('reason') == 'compiler-message' else r
            for r in records if r.get('reason') == 'compiler-message'
            and r['message']['level'] == 'error' or r.get('level') == 'error']


def at(errors, row, marker=None):
    return [e for e in errors if (marker is None or marker in e['message'])
            and any(s['is_primary'] and s['file_name'].replace('\\', '/').endswith(row['file'])
                    and (s['line_start'], s['column_start'], s['line_end'], s['column_end']) ==
                    tuple(int(row[k]) for k in ['start_line', 'start_column', 'end_line', 'end_column'])
                    for s in e['spans'])]


def shifted(row, changes):
    """Attribute insertions preserve lines; account for same-line columns."""
    result = dict(row)
    for endpoint in ['start', 'end']:
        line, column = int(row[endpoint + '_line']), int(row[endpoint + '_column'])
        result[endpoint + '_column'] = column + sum(len(value) for ln, col, value in changes.get(row['file'], ()) if ln == line and col <= column)
    return result


def edits(scratch, label, changes):
    for file, insertions in changes.items():
        path, text = scratch.text(file)
        for line, column, value in sorted(insertions, reverse=True):
            pos = offset(text, line, column)
            text = text[:pos] + value + text[pos:]
        scratch.write(label, path, text)


def compile_group(scratch, label, descriptor, fixture_root):
    if descriptor == 'fixture':
        command = ['rustc', '+1.85.0', '--edition=2024', '--crate-type=lib',
                   '--emit=metadata', '--error-format=json', str(fixture_root),
                   '-o', str(scratch.directory / 'fixture.rmeta')]
    else:
        package, kind, binary, _ = descriptor
        command = ['cargo', '+1.85.0', 'check', '--offline', '--locked', '-p', package,
                   kind, '--message-format=json']
        if binary:
            command.append(binary)
        if scratch.features == 'no-default-features':
            command.append('--no-default-features')
    result = scratch.run(label, command)
    errors = diagnostics(result)
    assert result.returncode == 0 or errors, f'compiler failed without Rust errors: {label}'
    return result.returncode, errors


def run(workspace, directory, features):
    started = time.monotonic()
    rows = read_tsv(directory / 'methods.tsv')
    scratch = Scratch(workspace, directory, features)
    groups = {}
    for row in rows:
        descriptor = 'fixture' if row['population'] == 'fixture' else target(dict(row, caller=row['definition']))
        groups.setdefault(descriptor, []).append(row)
    fixture_root = scratch.root / 'methods-fixture.rs'
    fixture_original = 'mod fixture { #[path="' + str(scratch.root / 'crates/rum-stdlib/src/tests/retention_closure/fixtures/methods.rs') + '"] pub mod methods; }\n'
    fixture_root.write_text(fixture_original)
    results = []
    try:
        for group_number, (descriptor, claims) in enumerate(groups.items()):
            label = f'group-{group_number}'
            status, errors = compile_group(scratch, label + '-control', descriptor, fixture_root)
            assert status == 0 and not errors, f'compile-pass control failed: {descriptor}'
            print(f'METHOD_CONTROL {descriptor} claims={len(claims)} compile=pass', flush=True)
            callers = {}
            for row in claims:
                callers.setdefault(row['file'], set()).add((int(row['caller_line']), int(row['caller_column']), '#[deny(deprecated)] '))
            positive = {file: set(v) for file, v in callers.items()}
            for row in claims:
                marker = 'brief535:' + row['target']
                positive.setdefault(row['target_file'], set()).add((int(row['target_line']), int(row['target_column']), f'#[deprecated(note="{marker}")] '))
            edits(scratch, label + '-positive', positive)
            status, errors = compile_group(scratch, label + '-positive', descriptor, fixture_root)
            positive_results = {row['name']: bool(at(errors, shifted(row, positive), 'brief535:' + row['target'])) for row in claims}
            scratch.restore()
            # rustc suppresses deprecation on direct self recursion. This is
            # an instrument limitation, not permission to omit those sites.
            recursive = [r for r in claims if r['definition'] == r['target']]
            if recursive:
                signatures = {}
                for row in recursive:
                    signatures.setdefault(row['target_file'], set()).add((int(row['signature_line']), int(row['signature_column']), row['signature_prefix'] + '__brief535_probe: ()'))
                edits(scratch, label + '-recursive-signatures', signatures)
                _, signature_errors = compile_group(scratch, label + '-recursive-signatures', descriptor, fixture_root)
                for row in recursive:
                    observed = at(signature_errors, shifted(row, signatures))
                    positive_results[row['name']] = any((e.get('code') or {}).get('code') == 'E0061' and any(
                        s['file_name'].replace('\\', '/').endswith(row['target_file']) and
                        s['line_start'] <= int(row['signature_line']) <= s['line_end']
                        for child in e['children'] for s in child.get('spans', [])) for e in observed)
                print(f'METHOD_RECURSIVE_SIGNATURES {label} claims={len(recursive)}', flush=True)
                scratch.restore()
            # One distinct same-named definition per claimed target, all called
            # independently under deny. They are real definitions in source
            # copies, not a fabricated negative boolean or a missing mutation.
            unique = {row['target']: row['method'] for row in claims}
            controls = '\nmod __brief535_negative {\n'
            for i, (identity, method) in enumerate(sorted(unique.items())):
                controls += f'struct N{i}; impl N{i} {{ #[deprecated(note="negative:{identity}")] fn {method}(&self) {{}} }}\n'
                controls += f'#[deny(deprecated)] fn control_{i}(x: &N{i}) {{ x.{method}(); }}\n'
            controls += '}\n'
            edits(scratch, label + '-negative', callers)
            root = fixture_root if descriptor == 'fixture' else scratch.root / descriptor[3]
            original_root = root.read_text()
            root.write_text(original_root + controls)
            evidence = directory / 'mutations' / (label + '-negative-controls.rs')
            evidence.write_text(controls)
            status, negative_errors = compile_group(scratch, label + '-negative', descriptor, fixture_root)
            for identity in unique:
                assert any('negative:' + identity in e['message'] and any(s['is_primary'] and 'control_' in (s.get('text', [{}])[0].get('text', '')) for s in e['spans']) for e in negative_errors), f'unexercised negative {identity}'
            root.write_text(original_root)
            scratch.restore()
            for row in claims:
                positive_ok = positive_results[row['name']]
                negative = bool(at(negative_errors, shifted(row, callers)))
                agrees = agreement(positive_ok, negative)
                result = 'AGREEMENT' if agrees else 'DISAGREEMENT'
                results.append((row['name'], result, row['file'], row['start_line'], row['start_column'], row['target'], str(positive_ok), str(negative)))
                print(f"METHOD_ORACLE {row['name']} {result} {row['file']}:{row['start_line']}:{row['start_column']} target={row['target']} positive={positive_ok} negative_site_error={negative}", flush=True)
            # Challenge wrong identity and removal of the negative control
            # using real observed compiler errors, not synthetic observations.
            assert any(positive_results.values()), 'no observed method selection'
            assert not any(at(errors, r, 'brief535:deliberately_wrong_target') for r in claims), 'wrong target escaped marker guard'
            try:
                agreement(True, None)
            except AssertionError as error:
                assert 'negative control disabled' in str(error)
            else:
                raise AssertionError('missing negative escaped instrument')
            print(f'METHOD_INSTRUMENT {label} wrong_target=caught missing_negative=caught independent_negatives={len(unique)}', flush=True)
    finally:
        scratch.restore()
        scratch.verify_originals()
        with (directory / 'results.tsv').open('w', newline='') as stream:
            writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
            writer.writerow(('name', 'result', 'file', 'line', 'column', 'target', 'positive', 'negative'))
            writer.writerows(results)
        elapsed = time.monotonic() - started
        print(f'METHOD_COST claims={len(rows)} observed={len(results)} seconds={elapsed:.3f}', flush=True)
    disagreements = sum(r[1] == 'DISAGREEMENT' for r in results)
    print(f'METHOD_TOTAL agreements={len(results)-disagreements} disagreements={disagreements} coverage={len(results)}/{len(rows)}', flush=True)
    assert len(results) == len(rows) and disagreements == 0, 'method selection disagreements require investigation'


if __name__ == '__main__':
    run(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
