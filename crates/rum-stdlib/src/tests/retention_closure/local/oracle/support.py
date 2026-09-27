"""Invariant expression carriers and exact stable-MIR observations shared by local oracles."""
import re

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
            adjustment = c.get('adjustment')
            annotation = ': ' + adjustment.replace('rum_runtime::', 'crate::') if adjustment else ''
            result = '({ let ' + marker + annotation + ' = ' + expression + '; ' + marker + ' })'
            return '(*' + result + ')' if c['place'] else result
        return expr
    return f['signature'] + render(0, len(f['body']))


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
        'is_system_symbol_name': 'expr_runtime_helpers::is_system_symbol_name(',
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
