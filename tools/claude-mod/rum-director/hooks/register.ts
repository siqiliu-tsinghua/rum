import type { EngineInterface, Register } from 'claude-code'
import type { Pending } from '../types'

// The director's session discipline for the rum repository, as hooks:
// - a dispatch guard (a brief dispatch is one background Bash call, alone);
// - a commit guard (staging and committing are separate calls, the handoff
//   file is rewritten before every commit, and a source commit carries its
//   TODO.md and the brief's director gate);
// - a push guard (user ruling 2026-10-30: every commit is pushed at once, so a
//   commit or a dispatch is refused while HEAD is ahead of its upstream);
// - compaction after every commit once the context reaches FLOOR_TOKENS, with
//   the handoff file appended to the compacted conversation;
// - the same handoff instructions and re-injection on every other compaction.
// Pending work lives in $.state, so a reload of the mod does not lose it.
// Outside the rum repository every hook passes through.

const FLOOR_TOKENS = 200_000
const STATE_FILE = '.temp/director/state.md'
const RUM_MARKER = 'crates/rum-runtime/Cargo.toml'

export const INSTRUCTIONS = [
  'Director handoff for the rum repository. Keep: the active brief and its stage;',
  'the last commit hash; every ruling or decision the user made in this conversation,',
  'verbatim when short; questions still pending to the user; and what was promised',
  'next. Drop tool output and file contents that are on disk, naming their paths',
  `instead. The authoritative handoff is ${STATE_FILE}, appended after this`,
  'compaction. The user reads Chinese only: replies stay in Chinese.',
].join(' ')

// Where a command starts: the start, a separator or a substitution, then any
// VAR=value assignments and runners. An argument naming the dispatcher (a grep
// of it, a git add of it) is not a dispatch.
const AT_COMMAND = String.raw`(?:^|[\n;&|(\`]|\$\()\s*(?:[A-Za-z_][A-Za-z0-9_]*=\S*\s+)*(?:(?:bash|sh|exec|env|nohup|time)\s+)*`
const DISPATCH = new RegExp(`${AT_COMMAND}(?:\\S*/)?(?:dispatch-brief(?:\\.sh)?(?![\\w.-])|(?:cdx|codex)\\s+exec\\b)`)
const COMMIT = /(^|[\n;&|(]\s*)(cd\s+[^;&|\n]+&&\s*)?git(\s+-C\s+\S+)?\s+commit(?![\w-])/
const STAGE = /\bgit(\s+-C\s+\S+)?\s+(add|rm|mv|stage)\b/
const COMMIT_ALL = /\bgit(\s+-C\s+\S+)?\s+commit(?![\w-])[^\n]*\s(-a|-am|--all)\b/
const BRIEF = /^docs\/dev\/codex\/brief-\d+-[^/]*\.md$/
const GATE_SECTION = /^#{2,4} Director gate/m

// In $.state, not module variables: a reload of the mod (which an edit to it
// makes at the turn end) starts the module over and cancels its timers.
const PENDING = { plugin: 'rum-director', key: 'pending' } as const
const INJECT = { plugin: 'rum-director', key: 'injectAtTurnEnd' } as const

async function pendingOf($: EngineInterface): Promise<Pending | null> {
  const { value = null } = await $.state.get(PENDING)
  return value
}

// A heredoc's body is text (a commit message, a file), not commands: drop it
// before matching, keeping the line that opens it.
export function withoutHeredocBodies(command: string): string {
  const kept: string[] = []
  let end: string | undefined
  for (const line of command.split('\n')) {
    if (end !== undefined) {
      if (line.trim() === end) end = undefined
      continue
    }
    kept.push(line)
    end = /(?<!<)<<(?!<)-?\s*(['"]?)([A-Za-z_][A-Za-z0-9_]*)\1/.exec(line)?.[2]
  }
  return kept.join('\n')
}

// A quoted string is an argument (a grep pattern, a message), not commands:
// blank its body, so a `|` or a name inside it starts nothing. The body of
// `-c` (bash -c '...') is a command, so it is kept, its opening quote turned
// into `(` so that the command inside starts where AT_COMMAND looks. Only a
// shell's -c: `grep -c 'x'` is a count.
const SHELL_C = /(?:^|[\s;&|(/])(?:ba|z|da|k)?sh\s+(?:-[A-Za-z]+\s+)*-[A-Za-z]*c\s*$/
export function withoutQuotedText(command: string): string {
  let out = ''
  let i = 0
  while (i < command.length) {
    const c = command[i]
    if (c === '\\') {
      out += command.slice(i, i + 2)
      i += 2
      continue
    }
    if (c !== "'" && c !== '"') {
      out += c
      i++
      continue
    }
    let j = i + 1
    while (j < command.length && command[j] !== c) j += c === '"' && command[j] === '\\' ? 2 : 1
    const body = command.slice(i + 1, j)
    out += SHELL_C.test(out) ? `(${body} ` : ' '.repeat(body.length + 2)
    i = j + 1
  }
  return out
}

// The text the guards match: heredoc bodies dropped first (their delimiter
// may be quoted), then quoted text blanked.
export function commandText(full: string): string {
  return withoutQuotedText(withoutHeredocBodies(full))
}

export function isDispatch(full: string): boolean {
  const command = commandText(full)
  return DISPATCH.test(command) && !command.includes('DRY_RUN=1')
}

export function dispatchProblems(full: string, background: boolean): string[] {
  const command = commandText(full)
  if (!isDispatch(full)) return []
  const problems: string[] = []
  if (/&&|\|\||;|\n/.test(command)) problems.push('it is chained with other commands (&&, ||, ; or a newline)')
  if (/(^|[^|])\|(?!\|)/.test(command)) problems.push('it is piped (pipefail plus a closed pipe kills the dispatcher before it dispatches)')
  if (/(?<![>&])&(?!&)/.test(command)) problems.push("it is detached with a bare '&'")
  if (!background) problems.push('run_in_background is not true (a foreground dispatch dies at the Bash tool timeout)')
  return problems
}

export function stagingProblem(full: string): string | undefined {
  const command = commandText(full)
  if (STAGE.test(command) || COMMIT_ALL.test(command)) {
    return 'stage in its own Bash call first (git add ...), then commit in a separate call, so the guard reads the final staged set.'
  }
  return undefined
}

export function isCommit(command: string): boolean {
  return COMMIT.test(commandText(command))
}

async function root($: EngineInterface): Promise<string> {
  return $.session.root()
}

async function isRum($: EngineInterface): Promise<boolean> {
  return $.fs.exists(`${await root($)}/${RUM_MARKER}`)
}

async function git($: EngineInterface, args: string[]): Promise<string> {
  const ran = await $.process.run(['git', ...args], { cwd: await root($), timeoutMs: 30_000 })
  if (ran.exitCode !== 0) throw new Error(`git ${args.join(' ')} exited ${ran.exitCode}: ${ran.stderr.trim()}`)
  return ran.stdout
}

async function headOf($: EngineInterface): Promise<{ hash: string; seconds: number }> {
  const [hash = '', seconds = '0'] = (await git($, ['log', '-1', '--format=%H %ct'])).trim().split(' ')
  return { hash, seconds: Number(seconds) }
}

// HEAD's commits not on its upstream. A branch with no upstream is not
// checked (rev-list exits non-zero).
async function unpushed($: EngineInterface): Promise<string | undefined> {
  const ran = await $.process.run(['git', 'rev-list', '--count', '@{upstream}..HEAD'], { cwd: await root($), timeoutMs: 30_000 })
  const count = Number(ran.stdout.trim())
  if (ran.exitCode !== 0 || !(count > 0)) return undefined
  return `HEAD is ${count} commit(s) ahead of its upstream: push first (git push origin main, its own call), since every commit is pushed at once (user ruling 2026-10-30).`
}

export async function commitProblems($: EngineInterface): Promise<string[]> {
  const staged = (await git($, ['diff', '--cached', '--name-only'])).split('\n').filter(p => p !== '')
  if (staged.length === 0) return []
  const problems: string[] = []
  const ahead = await unpushed($)
  if (ahead !== undefined) problems.push(ahead)
  const head = await headOf($)
  const statePath = `${await root($)}/${STATE_FILE}`
  const stateTime = (await $.fs.exists(statePath)) ? (await $.fs.stat(statePath)).mtimeMs : 0
  if (stateTime <= head.seconds * 1000) {
    problems.push(
      `${STATE_FILE} has not been rewritten since the last commit (${head.hash.slice(0, 8)}): write the handoff first (brief and stage, this commit, the next step, open rulings, questions pending to the user).`,
    )
  }
  if (staged.some(p => p.startsWith('crates/'))) {
    if (!staged.includes('TODO.md')) problems.push('a source commit must carry TODO.md, aligned to what this commit lands.')
    let hasGate = false
    for (const brief of staged.filter(p => BRIEF.test(p))) {
      if (GATE_SECTION.test(await git($, ['show', `:${brief}`]))) {
        hasGate = true
        break
      }
    }
    if (!hasGate) problems.push('a source commit must carry its brief file with a "Director gate" section.')
  }
  return problems
}

async function memoryTouchedSince($: EngineInterface, seconds: number): Promise<boolean> {
  const dir = (await root($)).replace(/[^A-Za-z0-9]/g, '-')
  const ran = await $.process.run(
    ['sh', '-c', 'find "$HOME/.claude/projects/$0/memory" -maxdepth 1 -name "*.md" -newermt "@$1" | head -1', dir, String(seconds)],
    { timeoutMs: 10_000 },
  )
  return ran.stdout.trim() !== ''
}

async function injectState($: EngineInterface): Promise<boolean> {
  const path = `${await root($)}/${STATE_FILE}`
  const text = (await $.fs.exists(path)) ? await $.fs.read(path) : `(${STATE_FILE} is missing)`
  const appended = await $.session.append({
    message: {
      type: 'user',
      content: [{ type: 'text', text: `<director-state source="${STATE_FILE}">\n${text.trim()}\n</director-state>` }],
    },
  })
  return !(appended !== undefined && appended !== null && typeof appended === 'object' && 'deny' in appended && appended.deny !== undefined)
}

function wan(tokens: number | undefined): string {
  return tokens === undefined ? '?' : `${(tokens / 10_000).toFixed(1)}万`
}

async function compactAfterCommit($: EngineInterface, commit: Pending): Promise<void> {
  // Read the size again: another instance of this mod (a resumed session that
  // loads it twice) or a compaction in between may already have brought it down.
  const { context } = await $.session.usage()
  if (context.tokens !== undefined && context.tokens < FLOOR_TOKENS) {
    await $.state.set(PENDING, null)
    return
  }
  let result
  try {
    result = await $.session.compact({ instructions: INSTRUCTIONS })
  } catch {
    // A turn is running (a notification started one): retry at its end.
    return
  }
  await $.state.set(PENDING, null)
  if (result.skip !== undefined) {
    $.ui.toast(`rum-director: compaction after ${commit.hash} skipped: ${result.skip}`)
    return
  }
  if (!(await injectState($))) await $.state.set(INJECT, true)
  $.ui.toast(`rum-director: compacted after ${commit.hash}: ${wan(result.tokensBefore)} → ${wan(result.tokensAfter)}`)
}

export const register: Register = on => {
  on('tool.call', { tool: 'Bash' }, async ($, e, next) => {
    if (!(await isRum($))) return next(e)
    const command = e.command
    const dispatch = dispatchProblems(command, e.run_in_background === true)
    if (dispatch.length > 0) {
      return {
        deny: `rum-director dispatch guard: ${dispatch.join('; ')}. A dispatch is its own single Bash call with run_in_background: true; commit first, in a separate call. (DRY_RUN=1 is exempt.)`,
      }
    }
    if (isDispatch(command)) {
      const ahead = await unpushed($)
      if (ahead !== undefined) return { deny: `rum-director push guard: ${ahead}` }
    }
    if (!isCommit(command)) return next(e)
    const staging = stagingProblem(command)
    if (staging !== undefined) return { deny: `rum-director commit guard: ${staging}` }
    const problems = await commitProblems($)
    if (problems.length > 0) return { deny: `rum-director commit guard: ${problems.join(' ')}` }

    const before = await headOf($)
    const sourceStaged = (await git($, ['diff', '--cached', '--name-only'])).split('\n').some(p => p.startsWith('crates/'))
    const ran = await next(e)
    const after = await headOf($)
    if (after.hash !== before.hash) {
      await $.state.set(PENDING, { hash: after.hash.slice(0, 8) })
      if (sourceStaged && !(await memoryTouchedSince($, before.seconds))) {
        $.ui.toast('rum-director: a source commit landed and no memory file changed since the previous commit.')
      }
    }
    return ran
  }).catch(($, e, next) =>
    next.called ? next(e) : { deny: `rum-director: its Bash guard failed, so the call is refused. Check the guard (tools/claude-mod/rum-director).` },
  )

  on('turn.complete', async ($, e, next) => {
    const result = await next(e)
    if (e.agentId !== undefined || !(await isRum($))) return result
    const { context } = await $.session.usage()
    if (context.tokens !== undefined) $.ui.status(`ctx ${wan(context.tokens)}`)
    const { value: inject = false } = await $.state.get(INJECT)
    if (inject && (await injectState($))) await $.state.set(INJECT, false)
    const commit = await pendingOf($)
    if (commit !== null && !e.isAborted) {
      if (context.tokens === undefined || context.tokens < FLOOR_TOKENS) {
        await $.state.set(PENDING, null)
      } else {
        $.clock.after(500, () => {
          void compactAfterCommit($, commit)
        })
      }
    }
    return result
  })

  // A reload (or a restart that kept the session's state) cancelled the timer
  // of a compaction still owed: schedule it again. A turn still running makes
  // the compaction throw, and the next turn end retries it.
  on('session.start', async ($, e, next) => {
    const result = await next(e)
    if (!(await isRum($))) return result
    const commit = await pendingOf($)
    if (commit !== null) {
      $.clock.after(500, () => {
        void compactAfterCommit($, commit)
      })
    }
    return result
  })

  on('session.compact', async ($, e, next) => {
    if (e.agentId !== undefined || !(await isRum($))) return next(e)
    const instructions = e.instructions === undefined ? INSTRUCTIONS : `${e.instructions}\n\n${INSTRUCTIONS}`
    const result = await next({ ...e, instructions })
    if (result.skip === undefined && e.trigger !== 'precompute') {
      await $.state.set(PENDING, null)
      if (!(await injectState($))) await $.state.set(INJECT, true)
    }
    return result
  }).catch(($, e, next) => next(e))
}
