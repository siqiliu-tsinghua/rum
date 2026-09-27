"""Stable 1.85 observations, never execution, of copied real probe bodies.

MIR lets every named expression have a separately observed callee. Invariant
carriers prevent coercion from making a type claim true. All copies, logs,
mutants and build outputs live below target/brief-540; originals are hashed.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
workspace, directory, features = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
root = directory / 'workspace'
assert root.resolve().is_relative_to((workspace / 'target/brief-540').resolve())
root.mkdir(parents=True, exist_ok=True)
originals = {str(p.relative_to(workspace)): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in (workspace / 'crates').rglob('*.rs')}
for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']:
    shutil.copy2(workspace / name, root / name)
shutil.copytree(workspace / 'crates', root / 'crates', dirs_exist_ok=True,
                ignore=shutil.ignore_patterns('target', '__pycache__'))
file = root / 'crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs'
original = file.read_text()
functions = json.loads((directory / 'claims.json').read_text())
scaffold = '\nstruct __B540Freeze<T>(T, std::marker::PhantomData<fn(T) -> T>);\nfn __b540_exact<T>(_: &__B540Freeze<T>) {}\n'


def instrument(f, kind):
    claims = f['claims']
    spans = {(c['start'], c['end']): i for i, c in enumerate(claims)}
    def render(start, end, outer=None):
        inside = sorted((a, -b, i) for (a, b), i in spans.items()
                        if a >= start and b <= end and i != outer)
        text, at = '', start
        for a, minus_b, i in inside:
            b = -minus_b
            if a < at:
                continue
            text += f['body'][at:a] + wrap(i, render(a, b, i))
            at = b
        return text + f['body'][at:end]
    def wrap(i, expr):
        c = claims[i]
        marker = '__b540_' + kind + '_' + f['name'].replace('-', '_') + '_' + str(i)
        if kind in ('types', 'wrong') and c['exact']:
            expected = c['exact'].replace('rum_runtime::', 'crate::')
            if c['place']:
                expected, expr = '&' + expected, '&(' + expr + ')'
            if kind == 'wrong':
                expected = 'std::convert::Infallible'
            result = ('({ let ' + marker + ' = __B540Freeze(' + expr + ', std::marker::PhantomData); '
                      '__b540_exact::<' + expected + '>(&' + marker + '); ' + marker + '.0 })')
            return '(*' + result + ')' if c['place'] else result
        if kind == 'resolution' and c['targets']:
            expression = '&(' + expr + ')' if c['place'] else expr
            result = '({ let ' + marker + ' = ' + expression + '; ' + marker + ' })'
            return '(*' + result + ')' if c['place'] else result
        return expr
    return f['signature'] + render(0, len(f['body']))


def run(label, code, mir=False):
    file.write_text(original + scaffold + code)
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(directory / 'build')
    command = ['cargo', '+1.85.0', 'rustc', '--offline', '--locked', '-p', 'rum-runtime', '--tests', '--message-format=json']
    if features == 'no-default-features':
        command.append('--no-default-features')
    command.extend(['--', '--emit=mir' if mir else '--emit=metadata'])
    command = ['python3', str(workspace / 'scripts/run-bounded.py'), '--timeout', '240', '--', *command]
    result = subprocess.run(command, cwd=root, env=env, capture_output=True)
    evidence = directory / label
    evidence.mkdir(exist_ok=True)
    (evidence / 'command.json').write_text(json.dumps(command) + '\n')
    (evidence / 'stdout').write_bytes(result.stdout)
    (evidence / 'stderr').write_bytes(result.stderr)
    (evidence / 'status').write_text(str(result.returncode) + '\n')
    (evidence / 'instrumented.rs').write_text(file.read_text())
    if result.returncode:
        print(label + ' failed: ' + result.stderr.decode()[-4000:], flush=True)
    bodies = []
    if mir and result.returncode == 0:
        for path in (directory / 'build/debug/deps').glob('rum_runtime-*.mir'):
            data = path.read_text()
            if '__b540_' in data:
                (evidence / path.name).write_text(data)
                bodies.extend(re.split(r'(?m)(?=^fn )', data))
    return result, bodies


def observed(bodies, marker, indirect=False):
    found = [b for b in bodies if re.search(r'debug ' + re.escape(marker) + r' => ', b)]
    assert len(found) == 1, ('missing/ambiguous MIR expression marker', marker, len(found))
    body = found[0]
    binding = re.search(r'debug ' + re.escape(marker) + r' => ([^;]+);', body)[1]
    if not binding.startswith('_'):
        return [binding]
    definitions = {}
    for name, value in re.findall(r'(?m)^\s+(_\d+) = (.*);$', body):
        definitions.setdefault(name, []).append(value)
    seen = set()
    def follow(name):
        if name in seen:
            return []
        seen.add(name)
        output = []
        for value in definitions.get(name, []):
            is_call = ' -> [' in value or ' -> ' in value
            if is_call:
                output.append(value)
                if not indirect:
                    continue
            for child in re.findall(r'_\d+', value):
                output.extend(follow(child))
        return output
    return follow(re.search(r'_\d+', binding)[0])


def matches(target, observation):
    # Only these pinned rustc display aliases are normalized. Anchor the
    # complete owner/trait identity: a same-named method is not evidence.
    def normalize(s):
        for a, b in [('rum_core::expr::', 'rum_core::'), ('std::vec::Vec', 'Vec'),
                     ('std::clone::Clone', 'Clone'), ('std::cmp::PartialEq', 'PartialEq'),
                     ('std::ops::Deref', 'Deref'), ('std::ops::Index', 'Index')]:
            s = s.replace(a, b)
        return s
    t, o = normalize(target), normalize(observation)
    if target.startswith('<'):
        return o.startswith(t + '(')
    method = re.findall(r'::([A-Za-z_]+)(?:::<|$)', target)[-1]
    modes = [m for m in ['Unarmed', 'AccountingShadow'] if '::' + m in target]
    mode = modes[0] if modes else None
    if method in ('start', 'drive'):
        return bool(re.match(method + r'::<counter::(Unarmed|AccountingShadow)>\(', o))
    exact = {
        'small_integer': 'rum_core::Expr::small_integer(',
        'string': 'rum_core::Expr::string::<std::string::String>(',
        'repeat': 'std::str::<impl str>::repeat(',
        'is_some': 'Option::<usize>::is_some(',
        'len': 'Vec::<rum_core::Expr>::len(',
    }
    if method in exact:
        return o.startswith(exact[method])
    if mode is None:
        return False
    context = "accounting_mode::retained::RetentionContext::<'_, '_, counter::" + mode + '>'
    if method in ('reserve', 'push', 'finish_list'):
        return o.startswith(context + '::' + method + '(')
    if method in ('evaluate', 'resume_values'):
        module = 'bridge' if method == 'evaluate' else 'frozen'
        owner = context.replace('RetentionContext::<', 'RetentionContext<')
        return o.startswith(module + '::<impl ' + owner + '>::' + method + '(')
    if method == 'scoped':
        return o.startswith('accounting_mode::retained::scoped::<counter::' + mode + ', {closure@')
    if method == 'suspend':
        return o.startswith("CallbackFailure::<'_, '_, counter::" + mode +
                            '>::suspend::<accounting_mode::tests::ProbeContinuation, {closure@')
    adopt = "bridge::CallbackValue::<'_, '_, counter::" + mode + '>::adopt'
    if method == 'adopt':
        return o == adopt or o.startswith(adopt + '(')
    if method == 'and_then':
        return o.startswith("Result::<bridge::CallbackValue<'_, '_, counter::" + mode +
                            ">, CallbackFailure<'_, '_, counter::" + mode + ">>::and_then::<") and adopt in o
    return False


try:
    positive = '\n'.join(instrument(f, 'types') for f in functions)
    result, _ = run('types-positive', positive)
    assert result.returncode == 0, 'invariant type claims disagree with rustc'
    print('LOCAL540_TYPES positive=pass functions=7 both_modes=true invariant=true', flush=True)
    wrong = '\n'.join(instrument(f, 'wrong') for f in functions)
    result, _ = run('types-negative', wrong)
    assert result.returncode != 0 and b'E0308' in result.stdout + result.stderr, 'wrong-type control did not fire'
    rejected = set()
    for line in result.stdout.decode().splitlines():
        try:
            message = json.loads(line).get('message', {})
            if not isinstance(message, dict) or (message.get('code') or {}).get('code') != 'E0308':
                continue
            for span in message.get('spans', []):
                if span['is_primary']:
                    for text in span['text']:
                        highlighted = text['text'][text['highlight_start']-1:text['highlight_end']-1]
                        rejected.update(re.findall(r'__b540_wrong_\w+', highlighted))
        except (ValueError, KeyError):
            continue
    expected = {'__b540_wrong_' + f['name'].replace('-', '_') + '_' + str(i)
                for f in functions for i,c in enumerate(f['claims']) if c['exact']}
    assert expected <= rejected, ('wrong-type control missing a located claim', expected - rejected)
    print(f'LOCAL540_TYPES negative=E0308 located={len(expected)}', flush=True)
    names = sorted({re.findall(r'::([A-Za-z_]+)(?:::<|$)', target)[-1]
                    for f in functions for c in f['claims'] for target in c['targets']})
    negative = '\nstruct __B540Wrong;\nimpl __B540Wrong {\n'
    for name in names:
        negative += 'fn ' + name + '(self) -> u8 { 0 }\n'
    negative += '}\nfn __b540_negative() {\n'
    for name in names:
        negative += 'let __b540_negative_' + name + ' = __B540Wrong.' + name + '();\n'
    negative += '}\n'
    negative += '''
mod __b540_other_trait {
    pub trait Clone { fn clone(self) -> rum_core::Expr; }
    impl Clone for rum_core::Expr { fn clone(self) -> rum_core::Expr { self } }
}
fn __b540_other_clone(e: rum_core::Expr) {
    use __b540_other_trait::Clone;
    let __b540_negative_same_owner_clone = e.clone();
}
'''
    result, bodies = run('resolution', '\n'.join(instrument(f, 'resolution') for f in functions) + negative, True)
    assert result.returncode == 0, 'resolution instrumentation must compile'
    other_clone = observed(bodies, '__b540_negative_same_owner_clone')
    assert not any(matches('<rum_core::expr::Expr as std::clone::Clone>::clone', x) for x in other_clone), 'same owner/name but different trait fooled the oracle'
    rows = []
    for f in functions:
        for i, c in enumerate(f['claims']):
            for target in c['targets']:
                marker = '__b540_resolution_' + f['name'].replace('-', '_') + '_' + str(i)
                actual = observed(bodies, marker, 'Deref' in target)
                name = re.findall(r'::([A-Za-z_]+)(?:::<|$)', target)[-1]
                negative_actual = observed(bodies, '__b540_negative_' + name)
                agrees = any(matches(target, item) for item in actual)
                unrelated = any(matches(target, item) for item in negative_actual)
                rows.append((f['name'], c['site'], target, str(agrees), str(unrelated), repr(actual), repr(negative_actual)))
    (directory / 'resolutions.tsv').write_text('function\tsite\ttarget\tpositive\tnegative\tobserved\tcontrol\n' + '\n'.join('\t'.join(r) for r in rows) + '\n')
    bad = [r for r in rows if r[3:5] != ('True', 'False')]
    for row in bad:
        print('LOCAL540_DISAGREEMENT ' + repr(row), flush=True)
    assert not bad, 'located compiler resolution disagreement'
    print(f'LOCAL540_RESOLUTION claims={len(rows)} positive=pass same_name_negatives={len(names)} negative=pass', flush=True)
finally:
    file.write_text(original)
    for name, before in originals.items():
        assert hashlib.sha256((workspace / name).read_bytes()).hexdigest() == before, 'original source mutated: ' + name
