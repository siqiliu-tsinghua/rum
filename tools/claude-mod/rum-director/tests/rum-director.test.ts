import { expect, mock, test } from 'claude-code/testing'
import type { TestBody } from 'claude-code/testing'
import { dispatchProblems, isCommit, stagingProblem } from '../hooks/register.ts'

test('a dispatch must be one background call, alone', () => {
  expect(dispatchProblems('scripts/dispatch-brief.sh 631', true)).toEqual([])
  expect(dispatchProblems('DRY_RUN=1 scripts/dispatch-brief.sh 631 && ls', false)).toEqual([])
  expect(dispatchProblems('scripts/dispatch-brief.sh 631 2>&1 < /dev/null', true)).toEqual([])
  expect(dispatchProblems('git commit -q && scripts/dispatch-brief.sh 631', true).length).toBe(1)
  expect(dispatchProblems('scripts/dispatch-brief.sh 631 | tee x.log', true).length).toBe(1)
  expect(dispatchProblems('scripts/dispatch-brief.sh 631 &', true).length).toBe(1)
  expect(dispatchProblems('scripts/dispatch-brief.sh 631', false).length).toBe(1)
  expect(dispatchProblems('ls -la', false)).toEqual([])
})

test('the dispatcher is matched where a command starts, not as an argument', () => {
  expect(dispatchProblems('grep -n CDX_FORK scripts/dispatch-brief.sh', false)).toEqual([])
  expect(dispatchProblems('git add scripts/dispatch-brief.sh docs/x.md', false)).toEqual([])
  expect(dispatchProblems('sed -n 1,40p scripts/dispatch-brief.sh | head', false)).toEqual([])
  expect(dispatchProblems('echo "cdx exec"', false)).toEqual([])
  expect(dispatchProblems("git commit -q -F - <<'EOF'\nscripts/dispatch-brief.sh 631 next\nEOF", false)).toEqual([])
  expect(dispatchProblems('CDX_FORK=abc GRK_RESUME=1 scripts/dispatch-brief.sh 631', true)).toEqual([])
  expect(dispatchProblems('bash /home/remi/dev/rum/scripts/dispatch-brief.sh 631', false).length).toBe(1)
  expect(dispatchProblems('ls && ./scripts/dispatch-brief.sh 631', true).length).toBe(1)
  expect(dispatchProblems('x=$(scripts/dispatch-brief.sh 631)', false).length).toBe(1)
  expect(dispatchProblems('cdx exec -s workspace-write x', false).length).toBe(1)
})

test('quoted text starts no command; the body of -c does', () => {
  expect(dispatchProblems('grep -n "AT_COMMAND\\|dispatch-brief" register.ts | head', false)).toEqual([])
  expect(dispatchProblems('grep -n -E "WORKER|worker=|grk\\)" scripts/dispatch-brief.sh | head -20; ls', false)).toEqual([])
  expect(dispatchProblems("grep -c 'x|cdx exec' log", false)).toEqual([])
  expect(dispatchProblems('grep "a\\"|dispatch-brief" f', false)).toEqual([])
  expect(dispatchProblems("bash -c 'scripts/dispatch-brief.sh 632'", false).length).toBe(1)
  expect(dispatchProblems('sh -c "ls; scripts/dispatch-brief.sh 632"', true).length).toBe(1)
  expect(dispatchProblems("/bin/bash -lc 'scripts/dispatch-brief.sh 632'", false).length).toBe(1)
  expect(isCommit('git log --grep "x; git commit"')).toBe(false)
  expect(stagingProblem('git commit -q -m "then git add the rest"')).toBeUndefined()
})

test('a commit is recognised where a command starts, not inside text', () => {
  expect(isCommit('git commit -q -F - <<EOF\nmsg\nEOF')).toBe(true)
  expect(isCommit('cd /home/remi/dev/rum && git commit -q -F -')).toBe(true)
  expect(isCommit('git -C /x commit -m y')).toBe(true)
  expect(isCommit('echo "git commit"')).toBe(false)
  expect(isCommit('git commit-tree abc')).toBe(false)
  expect(isCommit('git log --oneline')).toBe(false)
  expect(isCommit("cat > x.sh <<'EOF'\ngit commit -m y\nEOF")).toBe(false)
})

test('staging and committing are separate calls', () => {
  expect(stagingProblem('git commit -q -F -')).toBeUndefined()
  expect(stagingProblem('git add TODO.md && git commit -q -F -')).toBeDefined()
  expect(stagingProblem('git commit -am "x"')).toBeDefined()
  expect(stagingProblem('git commit --all -m x')).toBeDefined()
  expect(stagingProblem("git commit -q -F - <<'EOF'\nstage with git add first\nEOF")).toBeUndefined()
})

// The world beneath the plugin: a repository whose answers each test sets.
type World = {
  staged: string[]
  heads: string[]
  stateMtimeMs: number
  briefText: string
  memoryTouched: boolean
  tokens: number
  tokensLater?: number
  calls: string[]
}

function world(overrides: Partial<World> = {}): World {
  return {
    staged: ['docs/x.md'],
    heads: ['aaaaaaaa1111 1000', 'bbbbbbbb2222 2000'],
    stateMtimeMs: 1_500_000,
    briefText: '# brief\n\n## Director gate of B3\n',
    memoryTouched: true,
    tokens: 250_000,
    calls: [],
    ...overrides,
  }
}

function seat(on: Parameters<TestBody>[1], w: World) {
  // HEAD is the first entry until the commit runs beneath the plugin, then the last.
  let committed = false
  on('session.root', () => ({ value: '/repo' }))
  on('fs.exists', () => ({ value: true }))
  on('fs.stat', () => ({ value: { kind: 'file', size: 10, mtimeMs: w.stateMtimeMs, isLink: false } }))
  on('fs.read', () => ({ value: 'brief 631, stage A; next: gate' }))
  let usageCalls = 0
  on('session.usage', () => {
    const tokens = usageCalls++ === 0 ? w.tokens : (w.tokensLater ?? w.tokens)
    return { value: { startedAt: 0, context: { tokens, window: 1_000_000, percent: tokens / 10_000 }, rateLimits: [], cost: { usd: 0 } } }
  })
  on('process.run', (_, e) => {
    const argv = (e as { argv: readonly string[] }).argv
    const line = argv.join(' ')
    w.calls.push(line)
    const ok = (stdout: string) => ({ value: { exitCode: 0, stdout, stderr: '', isStdoutTruncated: false, isStderrTruncated: false } })
    if (line.startsWith('git diff --cached --name-only')) return ok(w.staged.join('\n') + '\n')
    if (line.startsWith('git log -1')) {
      const head = (committed ? w.heads.at(-1) : w.heads[0]) ?? ''
      return ok(head + '\n')
    }
    if (line.startsWith('git show :')) return ok(w.briefText)
    if (argv[0] === 'sh') return ok(w.memoryTouched ? '/m/x.md\n' : '')
    return ok('')
  })
  on('tool.call', () => {
    committed = true
    return { result: {} }
  })
  on('turn.complete', () => ({ text: 'done' }))
  on('ui.status', () => ({ value: undefined }))
  on('ui.toast', () => ({ value: undefined }))
}

test('a dispatch chained after a commit is refused', async ($, on) => {
  const w = world()
  seat(on, w)
  const ran = await $.tool.call({ tool: 'Bash', command: 'git commit -q -F - && scripts/dispatch-brief.sh 631', run_in_background: true })
  expect(String(ran.deny)).toMatch(/dispatch guard/)
})

test('a commit with git add in the same call is refused', async ($, on) => {
  seat(on, world())
  const ran = await $.tool.call({ tool: 'Bash', command: 'git add TODO.md && git commit -q -F -' })
  expect(String(ran.deny)).toMatch(/stage in its own Bash call/)
})

test('a commit before the handoff file is rewritten is refused', async ($, on) => {
  seat(on, world({ stateMtimeMs: 900_000 }))
  const ran = await $.tool.call({ tool: 'Bash', command: 'git commit -q -F -' })
  expect(String(ran.deny)).toMatch(/state\.md has not been rewritten/)
})

test('a source commit needs TODO.md and the director gate', async ($, on) => {
  seat(on, world({ staged: ['crates/rum-runtime/src/a.rs', 'docs/dev/codex/brief-631-x.md'], briefText: '# brief\n' }))
  const ran = await $.tool.call({ tool: 'Bash', command: 'git commit -q -F -' })
  expect(String(ran.deny)).toMatch(/TODO\.md/)
  expect(String(ran.deny)).toMatch(/Director gate/)
})

test('an aligned source commit goes through', async ($, on) => {
  seat(on, world({ staged: ['crates/rum-runtime/src/a.rs', 'TODO.md', 'docs/dev/codex/brief-631-x.md'] }))
  const ran = await $.tool.call({ tool: 'Bash', command: 'git commit -q -F -' })
  expect(ran.deny).toBeUndefined()
})

const TURN = { answer: 'done', durationMs: 1, isAborted: false, turnId: 't1', reason: 'answer' } as const

test('a commit above the floor compacts at the turn end and appends the handoff', async ($, on) => {
  const clock = mock.clock(on)
  const rows = mock.session(on)
  const w = world()
  seat(on, w)
  const compacted: (string | undefined)[] = []
  on('session.compact', (_, e) => {
    compacted.push(e.instructions)
    return { messages: [{ role: 'user', text: 'summary', toolUses: [] }], tokensBefore: 250_000, tokensAfter: 70_000 }
  })
  await $.tool.call({ tool: 'Bash', command: 'git commit -q -F -' })
  await $.turn.complete(TURN)
  expect(compacted.length).toBe(0)
  await clock.advance(600)
  expect(compacted.length).toBe(1)
  expect(String(compacted[0])).toMatch(/Director handoff/)
  const text = JSON.stringify(rows.appended())
  expect(text).toMatch(/director-state/)
  expect(text).toMatch(/brief 631, stage A/)
})

test('a commit below the floor does not compact', async ($, on) => {
  const clock = mock.clock(on)
  seat(on, world({ tokens: 150_000 }))
  let compacted = 0
  on('session.compact', () => {
    compacted += 1
    return { messages: [{ role: 'user', text: 'summary', toolUses: [] }] }
  })
  await $.tool.call({ tool: 'Bash', command: 'git commit -q -F -' })
  await $.turn.complete(TURN)
  await clock.advance(600)
  expect(compacted).toBe(0)
})

test('a failed commit (head unchanged) does not compact', async ($, on) => {
  const clock = mock.clock(on)
  seat(on, world({ heads: ['aaaaaaaa1111 1000'] }))
  let compacted = 0
  on('session.compact', () => {
    compacted += 1
    return { messages: [{ role: 'user', text: 'summary', toolUses: [] }] }
  })
  await $.tool.call({ tool: 'Bash', command: 'git commit -q -F -' })
  await $.turn.complete(TURN)
  await clock.advance(600)
  expect(compacted).toBe(0)
})

test('a compaction that another instance already made is not repeated', async ($, on) => {
  const clock = mock.clock(on)
  seat(on, world({ tokens: 250_000, tokensLater: 70_000 }))
  let compacted = 0
  on('session.compact', () => {
    compacted += 1
    return { messages: [{ role: 'user', text: 'summary', toolUses: [] }] }
  })
  await $.tool.call({ tool: 'Bash', command: 'git commit -q -F -' })
  await $.turn.complete(TURN)
  await clock.advance(600)
  expect(compacted).toBe(0)
})

test('a compaction an earlier copy left owed is made after a reload', async ($, on) => {
  // What the copy before the reload stored stays; its timer did not.
  const clock = mock.clock(on)
  seat(on, world())
  let seeded = false
  on('state.get', (_, e, next) => {
    if (seeded || e.key !== 'pending') return next(e)
    seeded = true
    return { value: { value: { hash: 'abcd1234' }, version: 1 } }
  })
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  let compacted = 0
  on('session.compact', () => {
    compacted += 1
    return { messages: [{ role: 'user', text: 'summary', toolUses: [] }] }
  })
  await $.session.start({ cwd: '/repo', surface: 'terminal', isInteractive: true })
  await clock.advance(600)
  expect(compacted).toBe(1)
})
