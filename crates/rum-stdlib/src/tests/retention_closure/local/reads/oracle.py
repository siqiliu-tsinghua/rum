"""Observe the real option read sites with stable rustc, never admit their macros."""
import hashlib
import copy
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'oracle'))
from support import instrument, observed, matches
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'compiler_oracle'))
from scratch import refresh_crates

workspace, directory, features = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
root = directory / 'workspace'
assert root.resolve().is_relative_to((workspace / '.temp/brief-543').resolve())
root.mkdir(parents=True, exist_ok=True)
originals = {str(p.relative_to(workspace)): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in (workspace / 'crates').rglob('*.rs')}
for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']:
    shutil.copy2(workspace / name, root / name)
stale = root / 'crates/rum-stdlib/src/tests/retention_closure/local/__b542_stale.rs'
assert not (workspace / stale.relative_to(root)).exists()
stale.parent.mkdir(parents=True, exist_ok=True)
stale.write_text('// obsolete compiler input\n')
refresh_crates(workspace, root)
assert not stale.exists(), 'stale copied source survived refresh'
print('READ542_SCRATCH stale_input=removed', flush=True)
file = root / 'crates/rum-runtime/src/runtime/options/core.rs'
original = file.read_text()
functions = json.loads((directory / 'claims.json').read_text())
scaffold = '\nstruct __B540Freeze<T>(T, std::marker::PhantomData<fn(T) -> T>);\nfn __b540_exact<T>(_: &__B540Freeze<T>) {}\n'


def run(label, code, mir=False, execute=False):
    file.write_text(original + scaffold + code)
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(directory / 'build')
    command = ['cargo', '+1.85.0', 'test' if execute else 'rustc', '--offline', '--locked', '-p', 'rum-runtime']
    if features == 'no-default-features':
        command.append('--no-default-features')
    if execute:
        command += ['__b542_contract_controls', '--', '--nocapture']
    else:
        command += ['--tests', '--message-format=json', '--', '--emit=mir' if mir else '--emit=metadata']
    command = ['python3', str(workspace / 'scripts/run-bounded.py'), '--timeout', '240', '--', *command]
    result = subprocess.run(command, cwd=root, env=env, capture_output=True, stdin=subprocess.DEVNULL)
    evidence = directory / label
    evidence.mkdir(exist_ok=True)
    (evidence / 'command.json').write_text(json.dumps(command) + '\n')
    (evidence / 'stdout').write_bytes(result.stdout)
    (evidence / 'stderr').write_bytes(result.stderr)
    (evidence / 'status').write_text(str(result.returncode) + '\n')
    (evidence / 'instrumented.rs').write_text(file.read_text())
    bodies = []
    if mir and result.returncode == 0:
        for path in (directory / 'build/debug/deps').glob('rum_runtime-*.mir'):
            data = path.read_text()
            if '__b542_option_reads' in data:
                (evidence / path.name).write_text(data)
                bodies.extend(re.split(r'(?m)(?=^fn )', data))
    return result, bodies


def instrumented(kind):
    return '\nimpl Session {\n' + '\n'.join(instrument(f, kind) for f in functions) + '\n}\n'


def errors(result, code):
    messages = []
    for line in result.stdout.decode().splitlines():
        try:
            message = json.loads(line).get('message', {})
        except ValueError:
            continue
        if isinstance(message, dict) and (message.get('code') or {}).get('code') == code:
            messages.append(message)
    return messages


try:
    result, _ = run('types-positive', instrumented('types'))
    assert result.returncode == 0, 'invariant option types disagree: ' + result.stdout.decode()[-12000:]
    result, _ = run('types-negative', instrumented('wrong'))
    rejected = set()
    for message in errors(result, 'E0308'):
        for span in message['spans']:
            if span['is_primary']:
                for text in span['text']:
                    highlighted = text['text'][text['highlight_start']-1:text['highlight_end']-1]
                    rejected.update(re.findall(r'__b540_wrong_\w+', highlighted))
    expected = {'__b540_wrong_' + f['name'].replace('-', '_') + '_' + str(i)
                for f in functions for i, c in enumerate(f['claims']) if c['exact']}
    assert result.returncode != 0 and expected <= rejected, ('unlocated type negative', expected-rejected)
    print(f'READ542_TYPES positive=pass invariant=true negative=E0308 located={len(expected)}', flush=True)

    # Both wrong field types and an extra borrow are plausible coercion
    # mistakes. Require the invariant carrier to reject those exact mistakes,
    # in addition to the all-claims Infallible negative above.
    variants, expected = [], set()
    field_types = ['&rum_core::expr::SharedExpr', '&rum_core::expr::SharedArgs']
    for kind in ['fields', 'references']:
        f = copy.deepcopy(functions[0])
        f['name'] += '_' + kind
        f['signature'] = f['signature'].replace('__b542_option_reads', '__b542_' + kind)
        for i, claim in enumerate(f['claims']):
            if claim['exact'] in field_types:
                claim['exact'] = ('&' + claim['exact'] if kind == 'references'
                                  else field_types[1 - field_types.index(claim['exact'])])
                expected.add('__b540_types_' + f['name'] + '_' + str(i))
        variants.append(instrument(f, 'types'))
    result, _ = run('field-reference-negatives', 'impl Session {\n' + '\n'.join(variants) + '\n}')
    rejected = set()
    for message in errors(result, 'E0308'):
        for span in message['spans']:
            if span['is_primary']:
                for text in span['text']:
                    highlight = text['text'][text['highlight_start']-1:text['highlight_end']-1]
                    rejected.update(re.findall(r'__b540_types_\w+', highlight))
    assert expected and result.returncode != 0 and expected <= rejected, ('field/reference control', expected-rejected)
    print(f'READ542_FIELD_TYPES swapped_fields=E0308 extra_reference=E0308 located={len(expected)}', flush=True)

    names = sorted({re.findall(r'::([A-Za-z_]+)(?:::<|$)', target)[-1]
                    for f in functions for c in f['claims'] for target in c['targets']})
    negative = '\nstruct __B542Wrong;\nimpl __B542Wrong {\n'
    for name in names:
        negative += 'fn ' + name + '(self) -> u8 { 0 }\n'
    negative += '}\nfn __b542_negative() {\n'
    for name in names:
        negative += 'let __b542_negative_' + name + ' = __B542Wrong.' + name + '();\n'
    negative += '}\n'
    negative += '''
mod __b542_other_trait {
    pub trait Clone { fn clone(self) -> rum_core::Expr; }
    impl Clone for rum_core::Expr { fn clone(self) -> rum_core::Expr { self } }
}
fn __b542_other_clone(e: rum_core::Expr) {
    use __b542_other_trait::Clone;
    let __b542_negative_same_owner_clone = e.clone();
}
'''
    result, bodies = run('resolution', instrumented('resolution') + negative, mir=True)
    assert result.returncode == 0, 'option resolution instrumentation did not compile: ' + result.stdout.decode()[-12000:]
    assert not any(matches('<rum_core::expr::Expr as std::clone::Clone>::clone', x)
                   for x in observed(bodies, '__b542_negative_same_owner_clone'))
    rows = []
    for f in functions:
        for i, claim in enumerate(f['claims']):
            for target in claim['targets']:
                marker = '__b540_resolution_' + f['name'].replace('-', '_') + '_' + str(i)
                actual = observed(bodies, marker, 'Deref' in target)
                name = re.findall(r'::([A-Za-z_]+)(?:::<|$)', target)[-1]
                unrelated = observed(bodies, '__b542_negative_' + name)
                rows.append((claim['site'], target, str(any(matches(target, x) for x in actual)),
                             str(any(matches(target, x) for x in unrelated)), repr(actual), repr(unrelated)))
    (directory / 'resolutions.tsv').write_text('site\ttarget\tpositive\tnegative\tobserved\tcontrol\n' +
        '\n'.join('\t'.join(row) for row in rows) + '\n')
    bad = [row for row in rows if row[2:4] != ('True', 'False')]
    assert not bad, ('located option resolution disagreement', bad)
    print(f'READ542_RESOLUTION claims={len(rows)} positive=pass same_name_negatives={len(names)} negative=pass competing_clone=distinct', flush=True)

    controls = '''
#[test]
fn __b542_contract_controls() {
    use rum_core::SharedExpr;
    use std::ops::Deref;
    let call = Expr::call(Expr::system_symbol("RuleDelayed"), vec![Expr::small_integer(1)]);
    let shared = SharedExpr::new(call);
    let actual = <SharedExpr as Deref>::deref(&shared);
    assert!(std::ptr::eq(actual, shared.as_ref()));
    let Expr::Call { head, args } = actual else { panic!("call control"); };
    assert!(!std::ptr::eq(actual, head.as_ref()));
    assert!(is_system_symbol_name(head, "RuleDelayed"));
    assert!(!is_system_symbol_name(head, "Rule"));
    assert!(!is_system_symbol_name(&Expr::contextual_symbol("Global`", "RuleDelayed"), "RuleDelayed"));
    assert!(!is_system_symbol_name(&Expr::small_integer(1), "RuleDelayed"));
    assert!(!is_system_symbol_name(actual, "RuleDelayed"));
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].clone(), Expr::small_integer(1));
    let count = std::cell::Cell::new(0);
    let skipped = false && { count.set(count.get()+1); true };
    assert!(!skipped);
    assert_eq!(count.get(), 0);
    let taken = true && { count.set(count.get()+1); true };
    assert!(taken);
    assert_eq!(count.get(), 1);
}
'''
    result, _ = run('contracts-positive', controls, execute=True)
    assert result.returncode == 0 and b'1 passed; 0 failed' in result.stdout, 'concrete read controls failed'
    result, _ = run('contracts-negative', controls.replace('assert!(std::ptr::eq(actual, shared.as_ref()))',
                                                            'assert!(!std::ptr::eq(actual, shared.as_ref()))'), execute=True)
    assert result.returncode != 0 and b'__b542_contract_controls' in result.stdout, 'pointer control never fired'
    result, _ = run('helper-negative', controls.replace('assert!(is_system_symbol_name(head, "RuleDelayed"))',
                                                        'assert!(!is_system_symbol_name(head, "RuleDelayed"))'), execute=True)
    assert result.returncode != 0 and b'__b542_contract_controls' in result.stdout, 'helper control never fired'
    for label, code, diagnostic in [
        ('nondiverging-else', 'fn __b542_bad(e: &Expr) { let Expr::Call { head, args } = e else {}; }', 'E0308'),
        ('borrowed-partial-move', 'fn __b542_bad(e: &Expr) { let Expr::Call { head, mut args } = e else { return; }; }', 'E0507'),
        ('rhs-move', 'fn __b542_bad(raw: Vec<u8>, flag: bool) { flag && { let retired = raw; true }; drop(raw); }', 'E0382'),
    ]:
        result, _ = run(label, code)
        found = errors(result, diagnostic)
        assert result.returncode != 0 and found, (label, 'negative never fired')
        assert any(span['is_primary'] and span['line_start'] > len(original.splitlines())
                   for message in found for span in message['spans']), (label, 'unlocated failure')
        print(f'READ542_RUST_CONTROL {label} negative={diagnostic} located=true', flush=True)
    print('READ542_CONTRACTS helper_table=pass pointer_identity=pass bool_branches=pass pointer_negative=fail helper_negative=fail', flush=True)
    result, _ = run('restored-controls', instrumented('types') + controls, execute=True)
    assert result.returncode == 0 and b'1 passed; 0 failed' in result.stdout, 'restored controls failed'
    print('READ542_RESTORED invariant_types=pass contracts=pass after_all_negatives=true', flush=True)
finally:
    file.write_text(original)
    for name, before in originals.items():
        assert hashlib.sha256((workspace / name).read_bytes()).hexdigest() == before, 'original source mutated: ' + name
