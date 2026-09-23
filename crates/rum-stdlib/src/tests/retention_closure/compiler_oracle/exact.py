"""A1 facts: invariant type probes, stable MIR identities, same-name controls.

No analyzed function is executed. The standalone crate consists of verbatim
compiled fixture copies. Production probes use the existing scratch harness.
Every observation and mutation survives under target/brief-534/oracle-exact.
"""
import csv
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
from scratch import Scratch, offset, read_tsv


def freeze(expression, expected):
    # Keep the inferred value inside the invariant carrier; don't use it twice.
    return ('({ struct __A1Frozen<T>(T, std::marker::PhantomData<fn(T) -> T>); '
            'fn __a1_check_frozen<T>(_: &__A1Frozen<T>) {} '
            f'let __a1_frozen = __A1Frozen({expression}, std::marker::PhantomData); '
            f'__a1_check_frozen::<{expected}>(&__a1_frozen); __a1_frozen.0 }})')


class Fixture:
    def __init__(self, workspace, directory):
        self.workspace, self.directory = workspace, directory
        self.root = directory / 'fixture'
        self.root.mkdir(parents=True, exist_ok=True)
        self.original = {}
        base = 'crates/rum-stdlib/src/tests/retention_closure/fixtures/'
        for name in ['accepted.rs', 'constructors.rs']:
            text = (workspace / base / name).read_text()
            self.original[base + name] = text
            (self.root / name).write_text(text)
        (self.root / 'lib.rs').write_text(
            '#![allow(dead_code, non_snake_case, non_camel_case_types)]\n'
            'extern crate self as fixture;\n'
            'pub mod accepted;\npub mod constructors;\npub mod negative;\n')
        self.negative = ''

    def write(self, file, text):
        (self.root / Path(file).name).write_text(text)

    def restore(self):
        for file, text in self.original.items():
            self.write(file, text)
        (self.root / 'negative.rs').write_text(self.negative)

    def compile(self, label):
        command = ['python3', str(self.workspace / 'scripts/run-bounded.py'), '--timeout', '60', '--',
                   'rustc', '+1.85.0', '--edition=2024', '--crate-name=fixture', '--crate-type=lib',
                   '--error-format=json', '--emit=metadata,mir', '-o', str(self.directory / 'fixture-output'),
                   str(self.root / 'lib.rs')]
        result = subprocess.run(command, capture_output=True)
        evidence = self.directory / label
        evidence.mkdir(exist_ok=True)
        (evidence / 'command.json').write_text(json.dumps(command) + '\n')
        (evidence / 'stdout').write_bytes(result.stdout)
        (evidence / 'stderr').write_bytes(result.stderr)
        (evidence / 'status').write_text(str(result.returncode) + '\n')
        for file in self.root.glob('*.rs'):
            shutil.copy2(file, evidence / file.name)
        errors = []
        for line in result.stderr.splitlines():
            if line.startswith(b'{'):
                item = json.loads(line)
                if item.get('level') == 'error':
                    errors.append(item)
        assert result.returncode == 0 or errors, f'compiler failed without Rust errors: {label}'
        mir = self.directory / 'fixture-output.mir'
        if result.returncode == 0:
            assert mir.exists(), 'stable MIR output missing'
            shutil.copy2(mir, evidence / 'output.mir')
        return result.returncode, errors

    def at(self, errors, row, code=None):
        return [e for e in errors if (code is None or (e.get('code') or {}).get('code') == code)
                and any(s['is_primary'] and Path(s['file_name']).name == Path(row['file']).name
                        and s['line_start'] == int(row['start_line']) for s in e['spans'])]

    def expression(self, row, expected):
        text = self.original[row['file']]
        start = offset(text, row['start_line'], row['start_column'])
        end = offset(text, row['end_line'], row['end_column'])
        if row['mode'] == 'place':
            # Probe the existing enclosing borrow, preserving place semantics.
            assert text[start - 1] == '&', 'place probe needs its checked enclosing borrow'
            start -= 1
            expected = '&' + expected
        self.write(row['file'], text[:start] + freeze(text[start:end], expected) + text[end:])


def run(workspace, directory, features):
    fixture = Fixture(workspace, directory)
    type_rows = read_tsv(directory / 'types.tsv')
    resolution_rows = read_tsv(directory / 'resolutions.tsv')
    names = sorted({r['target'].split('::')[-1] for r in resolution_rows})
    # The same-name negatives are real independent definitions with their own
    # live control calls. They never replace the selected namespace.
    fixture.negative = '\n'.join(f'pub fn {n}() {{}}\npub fn witness_{n}() {{ {n}(); }}' for n in names) + '\n'
    fixture.restore()
    results = []
    try:
        status, _ = fixture.compile('fixture-baseline')
        assert status == 0, 'every control must compile before it is analyzed'
        print('A1_ORACLE_BASELINE stable=1.85.0 compiled=true', flush=True)
        negative_errors = {}
        for name in names:
            text = fixture.negative.replace(f'fn {name}()', f'fn {name}(__probe: ())')
            (fixture.root / 'negative.rs').write_text(text)
            status, errors = fixture.compile('negative-' + name)
            fixture.restore()
            assert status != 0 and any((e.get('code') or {}).get('code') == 'E0061'
                                      and any(s['is_primary'] and Path(s['file_name']).name == 'negative.rs'
                                              for s in e['spans']) for e in errors), 'negative has no live witness'
            negative_errors[name] = errors

        # Invariant unification checks each recorded type, including field and
        # pattern-derived uses. No hand-pruned sample and no coercion assertion.
        for row in type_rows:
            observations = {}
            for mode, expected in [('exact', row['exact_type']), ('wrong', '__Wrong')]:
                if mode == 'wrong':
                    expected = 'std::convert::Infallible'
                fixture.expression(row, expected)
                status, errors = fixture.compile(row['name'] + '-' + mode)
                fixture.restore()
                observations[mode] = status, bool(fixture.at(errors, dict(row, start_line=row['end_line']), 'E0308'))
            assert observations['wrong'][0] != 0 and observations['wrong'][1], f'wrong type not caught at {row}'
            # All same-name mutation passes must leave the type site error-free.
            negative = any(fixture.at(errors, row) for errors in negative_errors.values())
            agreed = observations['exact'][0] == 0 and not negative
            result = 'AGREEMENT' if agreed else 'DISAGREEMENT'
            results.append(('type', row['name'], result, row['file'], row['start_line'], row['exact_type']))
            print(f"A1_ORACLE_TYPE {row['name']} {result} site={row['file']}:{row['start_line']} exact={row['exact_type']} wrong=E0308 negative_site_error={negative}", flush=True)

        # Name each original callee expression, preserving its lexical lookup.
        # rustc's stable MIR prints the resolved function-item identity in the
        # debug binding. Enum constructor items are distinguishable from a
        # same-named ordinary function returning the same Option/Result type.
        edits = {}
        for row in resolution_rows:
            if row['kind'] == 'unit-variant':
                continue
            text = fixture.original[row['file']]
            start = offset(text, row['start_line'], row['start_column'])
            end = offset(text, row['end_line'], row['end_column'])
            marker = '__a1_' + row['name']
            edits.setdefault(row['file'], []).append((start, end, f'({{ let {marker} = {text[start:end]}; {marker} }})'))
        for file, changes in edits.items():
            text = fixture.original[file]
            for start, end, replacement in sorted(changes, reverse=True):
                text = text[:start] + replacement + text[end:]
            fixture.write(file, text)
        status, errors = fixture.compile('resolved-items')
        fixture.restore()
        assert status == 0, 'callee-item instrumentation must preserve typing'
        mir = (directory / 'resolved-items/output.mir').read_text()
        for row in resolution_rows:
            name = row['target'].split('::')[-1]
            negative = bool(fixture.at(negative_errors[name], row))
            if row['kind'] == 'unit-variant':
                # A unit variant has no callee item. Its exact nominal type is
                # independently frozen at this site, and MIR records None.
                positive = any(r['file'] == row['file'] and r['start_line'] == row['start_line']
                               for r in type_rows) and '::None' in mir
                evidence = 'unit-variant+invariant-type'
            else:
                matches = re.findall(r'debug __a1_' + re.escape(row['name']) + r' => ([^;]+);', mir)
                evidence = '|'.join(matches)
                if row['kind'] == 'constructor':
                    family = 'Option' if name == 'Some' else 'Result'
                    positive = any(re.match(family + r'(?:::<.*>)?::' + name + r'$', item) for item in matches)
                else:
                    expected = row['target'].removeprefix('fixture::')
                    positive = any(item == expected or item.startswith(expected + '::<') for item in matches)
                    text = fixture.original[row['target_file']]
                    at = offset(text, row['target_line'], row['target_column'])
                    assert text[at] == ')'
                    addition = '' if text[at-1] == '(' else ', '
                    fixture.write(row['target_file'], text[:at] + addition + '__probe: ()' + text[at:])
                    status, errors = fixture.compile(row['name'] + '-target')
                    fixture.restore()
                    positive = status != 0 and bool(fixture.at(errors, row, 'E0061'))
            assert negative is not None, 'negative control disabled'
            result = 'AGREEMENT' if positive and not negative else 'DISAGREEMENT'
            results.append(('resolution', row['name'], result, row['file'], row['start_line'], row['target']))
            print(f"A1_ORACLE_RESOLUTION {row['name']} {result} site={row['file']}:{row['start_line']} target={row['target']} negative=negative::{name} negative_site_error={negative} MIR={evidence}", flush=True)

        # Real production copies: no fixture stand-ins for the legacy frontier.
        production_dir = directory / 'production'
        production_dir.mkdir(exist_ok=True)
        scratch = Scratch(workspace, production_dir, features)
        try:
            status, _ = scratch.compile('baseline')
            assert status == 0, 'production scratch baseline must compile'
            for row in read_tsv(directory / 'production.tsv'):
                # Mutate a real different same-name definition in this source
                # copy, with a live witness on the appended final source line.
                path, original = scratch.text(row['file'])
                name = row['target'].split('::')[-1]
                control = f' mod __a1_negative {{ fn {name}() {{}} fn control() {{ {name}(); }} }}'
                base = original[:-1] + control + '\n'
                scratch.write(row['name'] + '-negative-baseline', path, base)
                status, errors = scratch.compile(row['name'] + '-negative-baseline')
                assert status == 0, 'production negative control must first compile'
                scratch.restore()
                path, _ = scratch.text(row['file'])
                scratch.write(row['name'] + '-negative', path, base.replace(f'fn {name}()', f'fn {name}(__probe: ())'))
                status, errors = scratch.compile(row['name'] + '-negative')
                scratch.restore()
                control_line = original.count('\n')
                assert status != 0 and scratch.at(errors, row['file'], control_line, 'E0061'), 'production negative has no live witness'
                negative = bool(scratch.at(errors, row['file'], row['start_line']))
                assert not negative, 'same-name production negative changed the claimed site'
                exact_ok = False
                for mode, expected in [('exact', row['exact_type']), ('wrong', 'std::convert::Infallible')]:
                    scratch.expression(row['name'] + '-' + mode, row, lambda e: freeze(e, expected))
                    status, errors = scratch.compile(row['name'] + '-' + mode)
                    scratch.restore()
                    if mode == 'wrong':
                        assert status != 0 and scratch.at(errors, row['file'], row['start_line'], 'E0308'), f'production wrong-type control missed {row}'
                    else:
                        exact_ok = status == 0
                result = 'AGREEMENT' if exact_ok else 'DISAGREEMENT'
                results.append(('production-type', row['name'], result, row['file'], row['start_line'], row['exact_type']))
                print(f"A1_ORACLE_PRODUCTION {row['name']} {result} site={row['file']}:{row['start_line']} target={row['target']} exact={row['exact_type']} wrong=E0308 negative_site_error={negative}", flush=True)
        finally:
            scratch.restore()
            scratch.verify_originals()
    finally:
        fixture.restore()
        with (directory / 'results.tsv').open('w', newline='') as stream:
            writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
            writer.writerow(('technique', 'name', 'result', 'file', 'line', 'claim'))
            writer.writerows(results)
    disagreements = sum(r[2] == 'DISAGREEMENT' for r in results)
    print(f'A1_ORACLE_TOTAL agreements={len(results)-disagreements} disagreements={disagreements}', flush=True)


if __name__ == '__main__':
    run(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
