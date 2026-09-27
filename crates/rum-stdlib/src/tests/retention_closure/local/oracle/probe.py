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
from support import instrument, observed, matches
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'compiler_oracle'))
from scratch import refresh_crates

workspace, directory, features = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
root = directory / 'workspace'
assert root.resolve().is_relative_to((workspace / 'target/brief-540').resolve())
root.mkdir(parents=True, exist_ok=True)
originals = {str(p.relative_to(workspace)): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in (workspace / 'crates').rglob('*.rs')}
for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']:
    shutil.copy2(workspace / name, root / name)
refresh_crates(workspace, root)
file = root / 'crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs'
original = file.read_text()
functions = json.loads((directory / 'claims.json').read_text())
scaffold = '\nstruct __B540Freeze<T>(T, std::marker::PhantomData<fn(T) -> T>);\nfn __b540_exact<T>(_: &__B540Freeze<T>) {}\n'



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
