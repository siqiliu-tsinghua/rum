"""Observe every census constructor identity using stable compiler output.

The compiler names the original callee item while retaining the call context.
MIR distinguishes a constructor item from an ordinary same-named function.
No function pointer is invoked and no expected-type inference is frozen. Same-name negatives have
independently exercised calls in a separate module in each compiled crate.
Inactive conditional alternatives are reported as disagreements, never pruned.
"""
import csv
import json
import os
from pathlib import Path
import re
import shutil
import sys

sys.dont_write_bytecode = True
from scratch import Scratch, offset, read_tsv


def target(row):
    package = Path(row['file']).parts[1]
    crate = row['caller'].split('::')[0]
    root = f'crates/{package}/src/'
    if '_bin_' in crate:
        binary = crate.split('_bin_', 1)[1].replace('_', '-')
        return package, '--bin', binary, root + 'bin/' + binary + '.rs'
    if crate.endswith('_bin'):
        return package, '--bin', package, root + 'main.rs'
    return package, '--lib', '', root + 'lib.rs'


def wrapper(row):
    # Naming the callee preserves the call's expected return type and its
    # argument coercion sites. Freezing the result would change those sites.
    name = '__a1_' + row['name']
    return ('({ let ' + name + ' = ', '; ' + name + ' })')


def observations(mir):
    # Scan each compiler output once, rather than once per source claim.
    observed = {}
    for name, value in re.findall(r'debug __a1_(corpus_\d+) => (.*);$', mir, re.MULTILINE):
        observed.setdefault(name, []).append(value)
    return observed


def compile_target(scratch, label, descriptor, mir):
    package, kind, binary, _ = descriptor
    command = ['cargo', '+1.85.0', 'rustc' if mir else 'check', '--offline', '--locked', '-p', package, kind]
    if binary:
        command.append(binary)
    command.append('--message-format=json')
    if scratch.features == 'no-default-features':
        command.append('--no-default-features')
    if mir:
        # Give this invocation its own output instead of choosing a cached
        # artifact by name or timestamp across feature configurations.
        command.extend(['--', '--emit=mir=' + str(scratch.directory / (label + '.mir'))])
    result = scratch.run(label, command)
    errors = []
    for line in result.stdout.splitlines():
        record = json.loads(line)
        if record.get('reason') == 'compiler-message' and record['message']['level'] == 'error':
            errors.append(record['message'])
    assert result.returncode == 0 or errors, f'no compiler diagnostics: {label}'
    return result.returncode, errors


def compile_tests(scratch, label, descriptor):
    # Only the unobserved cfg(test) sites need this lane. It compiles tests;
    # it never executes them or enables a permissive checker feature.
    # Stdlib tests include these two repository documents at compile time.
    for file in ['TODO.md', 'docs/reference/builtins.md']:
        destination = scratch.root / file
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(scratch.original / file, destination)
    flags = os.environ.get('RUSTFLAGS', '') + ' --emit=mir'
    command = ['env', 'RUSTFLAGS=' + flags, 'cargo', '+1.85.0', 'test', '--offline',
               '--locked', '--no-run', '--lib', '-p', descriptor[0], '--message-format=json']
    if scratch.features == 'no-default-features':
        command.append('--no-default-features')
    result = scratch.run(label, command)
    records = [json.loads(line) for line in result.stdout.splitlines()]
    errors = [r['message'] for r in records
              if r.get('reason') == 'compiler-message' and r['message']['level'] == 'error']
    assert result.returncode == 0 or errors, 'test compilation failed without diagnostics'
    mir = None
    if result.returncode == 0:
        artifacts = [r for r in records if r.get('reason') == 'compiler-artifact'
                     and r['target']['src_path'] == str(scratch.root / descriptor[3])
                     and r['profile']['test'] and r['executable']]
        assert len(artifacts) == 1, 'one current test artifact must identify its MIR'
        mir = Path(artifacts[0]['executable']).with_suffix('.mir').read_text()
    return result.returncode, errors, mir


def run(workspace, directory, features):
    rows = read_tsv(directory / 'claims.tsv')
    scratch = Scratch(workspace, directory, features)
    groups, edits, results = {}, {}, []
    failed_targets = []
    for row in rows:
        groups.setdefault(target(row), []).append(row)
        edits.setdefault(row['file'], []).append(row)
    try:
        for file, changes in edits.items():
            path, original = scratch.text(file)
            insertions = []
            for row in changes:
                start = offset(original, row['callee_start_line'], row['callee_start_column'])
                end = offset(original, row['callee_end_line'], row['callee_end_column'])
                before, after = wrapper(row)
                insertions.extend([(start, before), (end, after)])
            text = original
            for at, addition in sorted(insertions, reverse=True):
                text = text[:at] + addition + text[at:]
            scratch.write('instrumented', path, text)
        for index, (descriptor, claims) in enumerate(sorted(groups.items())):
            label = 'target_' + str(index)
            status, errors = compile_target(scratch, label + '-items', descriptor, True)
            if status:
                failed_targets.append(descriptor)
                # Never turn an instrument compilation failure into agreement.
                # Preserve each affected site as a located disagreement.
                for row in claims:
                    results.append((row['name'], 'DISAGREEMENT', row['file'], row['start_line'], row['target'], 'instrumented-target-did-not-compile'))
                continue
            mir = (directory / (label + '-items.mir')).read_text()
            (directory / (label + '.mir')).write_text(mir)
            observed = observations(mir)
            # A different same-named definition per constructor, with an
            # independent call, in this same successfully compiled target.
            root = scratch.root / descriptor[3]
            if root not in scratch.originals:
                scratch.originals[root] = root.read_text()
            marked = root.read_text()
            definitions = ' '.join(f'fn {name}() {{}} fn control_{name}() {{ {name}(); }}' for name in ['Some', 'Ok', 'Err'])
            control = ' #[allow(dead_code, non_snake_case)] mod __a1_negative { ' + definitions + ' }'
            # The control is spliced onto the last existing line on purpose:
            # scratch.write requires a mutation to preserve the file's line
            # count, so every span-anchored claim in the same compile stays
            # valid. The cost is that the last line must be able to carry it —
            # a trailing // comment would swallow the control and leave the
            # negative control silently unexercised.
            assert not marked.rstrip('\n').rsplit('\n', 1)[-1].lstrip().startswith('//'), \
                f'last line of {descriptor[3]} is a line comment; it would swallow the negative control'
            base = marked[:-1] + control + '\n'
            scratch.write(label + '-negative-baseline', root, base)
            status, errors = compile_target(scratch, label + '-negative-baseline', descriptor, False)
            assert status == 0, f'negative baseline changed compilation: {descriptor}'
            mutated = base
            for name in ['Some', 'Ok', 'Err']:
                mutated = mutated.replace(f'fn {name}()', f'fn {name}(__probe: ())')
            scratch.write(label + '-negative', root, mutated)
            status, errors = compile_target(scratch, label + '-negative', descriptor, False)
            root.write_text(marked)
            assert status != 0 and len(scratch.at(errors, descriptor[3], marked.count('\n'), 'E0061')) == 3, f'same-name negative controls not exercised: {descriptor}'
            missing = [row for row in claims if row['name'] not in observed]
            test_errors = None
            test_observed = {}
            if missing:
                status, test_errors, test_mir = compile_tests(scratch, label + '-cfg-test-items', descriptor)
                assert status == 0, f'cfg(test) instrumentation failed: {descriptor}'
                (directory / (label + '-cfg-test.mir')).write_text(test_mir)
                test_observed = observations(test_mir)
                scratch.write(label + '-cfg-test-negative', root, mutated)
                status, test_errors, _ = compile_tests(scratch, label + '-cfg-test-negative', descriptor)
                root.write_text(marked)
                assert status != 0 and len(scratch.at(test_errors, descriptor[3], marked.count('\n'), 'E0061')) == 3, 'cfg(test) negative controls have no witnesses'
            for row in claims:
                matches = (test_observed if row in missing else observed).get(row['name'], [])
                variant = row['target'].split('::')[-1]
                family = 'Option' if variant == 'Some' else 'Result'
                positive = any(re.fullmatch(r'(?:(?:std|core)::(?:option|result)::)?' + family + r'(?:::<.*>)?::' + variant, item) for item in matches)
                negative = bool(scratch.at(test_errors if row in missing else errors, row['file'], row['start_line']))
                reason = '|'.join(matches) if matches else 'not-emitted-under-selected-cfg'
                result = 'AGREEMENT' if positive and not negative else 'DISAGREEMENT'
                results.append((row['name'], result, row['file'], row['start_line'], row['target'], reason))
    finally:
        scratch.restore()
        scratch.verify_originals()
        with (directory / 'results.tsv').open('w', newline='') as stream:
            writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
            writer.writerow(('name', 'result', 'file', 'line', 'target', 'evidence'))
            writer.writerows(results)
    assert len(results) == len(rows), 'every claim needs an observation'
    for name, result, file, line, target_, evidence in results:
        print(f'A1_CORPUS {name} {result} site={file}:{line} target={target_} evidence={evidence} negative=separate-same-name-definition')
    disagreements = sum(r[1] == 'DISAGREEMENT' for r in results)
    print(f'A1_CORPUS_TOTAL agreements={len(results)-disagreements} disagreements={disagreements} claims={len(rows)}')
    assert not failed_targets, f'instrumented target compilation failures: {failed_targets}'


if __name__ == '__main__':
    run(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
