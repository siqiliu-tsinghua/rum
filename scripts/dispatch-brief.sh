#!/usr/bin/env bash
# Dispatch a brief for execution by a coding-agent worker.
#
#   scripts/dispatch-brief.sh <brief-id>
#
# <brief-id> may be a number (081), "brief-081", or a slug fragment; it resolves to
# docs/dev/codex/brief-<id>*.md. Runs the worker non-interactively (stdin from /dev/null)
# and writes the full report to .temp/logs/<worker>_brief_<num>.log inside the repo
# (git-ignored; /tmp is wiped on reboot, which once lost a brief's dispatch log).
#
# This does NOT touch git — commit the brief separately FIRST (keep commit and dispatch as
# two steps). Invoke this via the Bash tool with run_in_background:true so the run is
# harness-tracked; never append `&` to a dispatch, and never pipe this script's output.
# `set -euo pipefail` plus a closed downstream pipe (`| head -n`) kills it mid-header,
# BEFORE it dispatches, and the result — exit 0, no log file, no worker changes — reads
# exactly like the silent mid-run terminations recorded in TODO.md. The tell is a missing
# header line.
#
# WORKER SELECTION (env BRIEF_WORKER, default codex)
#
# BRIEF_RUST_LOG overrides the codex log filter (default `warn,codex_core=info`).
# codex emits only ERROR at its own default, which is why a mid-run stream
# termination has so far left no diagnostic in the log.
#
#   BRIEF_WORKER=codex   `cdx exec` — Codex. The default worker.
#   BRIEF_WORKER=cgl     `cgl -p`   — claude-code on a GLM backend. EMERGENCY FALLBACK for
#                                     when Codex has no quota. Not the normal path.
#   BRIEF_WORKER=kimi    `kimi -p`  — Kimi Code (K3), for when Codex has no quota. A plain
#                                     binary, run directly (see KIMI below).
#   BRIEF_WORKER=agy     `agy -p`   — Antigravity CLI (Gemini 4 Argon), introduced by the user
#                                     2026-10-17. A shell function (`with_proxy agy`) with the
#                                     same name as its binary (see AGY below).
#   BRIEF_WORKER=grk     `grk -p`   — Grok Build (grok-4.7), introduced by the user 2026-10-17.
#                                     A shell function (`with_proxy grok`) (see GRK below).
#
# Both are interactive-shell functions in the developer's own config (`with_proxy codex` /
# `with_proxy claude`), so neither is visible to a bash subshell — each runs through an
# interactive shell, which sources the rc file that defines it (giving both the function and
# the proxy env). Redirections stay on the outer call so the worker inherits them.
#
# Which shell holds those definitions, and what they are named, varies per host — zsh and
# `cgl` on the macOS box this workflow grew up on, fish and `glm` on the Linux one. The
# script therefore probes: for each candidate name in preference order (cdx, then codex;
# cgl, then glm) it asks each candidate shell whether it defines it, and dispatches through
# the first pair that answers yes. Name preference dominates shell preference on purpose —
# the wrappers are NOT interchangeable with the bare binary, since `codex` in place of `cdx`
# silently drops the proxy — and it exits with an error rather than substituting when
# nothing resolves. BRIEF_SHELL (shell) and CDX_BIN / CGL_BIN (command name) override the
# probe, and an override is authoritative: it becomes the only candidate.
#
# PRIVILEGE — the two workers fence differently, and they are NOT equivalent:
#
#   codex  Sandbox defaults to workspace-write (user ruling 2026-07-17: least privilege;
#          writes confined to the repo + tmp) and is an OS-level fence on WRITES, while
#          commands themselves are unrestricted. Override ONLY for briefs that truly need
#          process attach (the macOS `sample` profiler — the original 2026-06-21 reason for
#          full access):  CDX_SANDBOX=danger-full-access scripts/dispatch-brief.sh <id>
#
#   cgl    claude-code has NO OS-level write fence. The nearest equivalent is a tool-level
#          permission mode, so this script uses acceptEdits (file edits auto-approved) plus
#          an explicit --allowedTools list, which keeps the deny list in
#          ~/.claude-glm/settings.json (e.g. `Bash(rm:*)`) in force. That is a WEAKER fence
#          than codex's: writes are not confined to the repo by the OS. Escalate only when a
#          brief genuinely cannot proceed:
#            CGL_PERMISSION_MODE=bypassPermissions scripts/dispatch-brief.sh <id>
#          The brief's own Scope + Hard rules remain the inner fence for both workers.
#
#   kimi   `kimi -p` hard-codes the "Never Ask" permission mode whatever the configured
#          default, so every tool call is approved and there is no write fence at all.
#          Its `[permission]` deny rules are not a usable fence either: they live in the
#          developer's global config, and in 2.1.1 nothing visibly feeds them to a print-mode
#          session. This script therefore fences git itself: a refusing `git` shim goes
#          first on the worker's PATH (checked before dispatch), and every attempt is logged
#          to <log>.git-attempts with its parent command. It stops PATH lookups, which is
#          what an agent's `git status` is; an absolute /usr/bin/git still gets through.
#
#   agy    An OS-level fence, comparable to codex's workspace-write and stricter on the
#          network. `--sandbox` runs every agent command in Linux namespaces: the host is
#          read-only except paths granted by a `write_file(...)` allow rule, temporary
#          directories and build caches (~/.cargo), and there is no network.
#          `toolPermission: proceed-in-sandbox` auto-approves commands inside it. Anything
#          else that needs approval — a command asking to leave the sandbox, the file tool
#          writing outside a granted path, a URL — is denied in print mode, and the denial
#          ENDS the run with exit 0 and a `denied_actions` list in the result; this script
#          turns that into exit 3. The settings live in the developer's agy config, not
#          here, so the script refuses to dispatch unless they hold (see AGY below). The
#          git shim is installed as for kimi.
#
#   grk    An OS-level fence, the tightest of the five. `--sandbox rum-worker` applies a
#          kernel profile (Landlock and seccomp, bwrap for its deny list) to the whole grok
#          process before any tool runs, and nothing can relax it afterwards. The profile
#          is grok's `strict` plus the Rust toolchain read-only. Reads are confined to the
#          repo, system paths, ~/.grok and the toolchain. Writes go only to the repo,
#          ~/.grok/sessions and the temp directories. Child processes have no network, and
#          the repo's .git is denied outright. Tool approval is bypassed (--always-approve)
#          because a headless grok REFUSES an unapproved call and carries on with exit 0,
#          which silently skips the work. The kernel profile is the fence, not prompts. The
#          toolset is an explicit allowlist. Web search, subagents and the Claude Code and
#          Cursor compatibility layers are off; that layer would otherwise load the MCP
#          servers in ~/.claude.json, including a Wolfram engine the testing policy forbids.
#          The profile lives in the developer's grok config, so the script checks it and
#          then observes the applied fence before dispatching (see GRK below). The git shim
#          is installed as for kimi.
#
# KIMI
#
# kimi is a plain binary, so it runs directly from this script rather than through an
# interactive shell. If the login shell ever defines `kimi` as a function, dispatch refuses
# rather than bypass the wrapper. Reasoning effort is pinned per run through
# KIMI_MODEL_THINKING_EFFORT (KIMI_EFFORT, default max), and each session's own log
# (~/.kimi-code/sessions/wd_<repo>/<session>/logs/kimi-code.log, `llm config` line) records
# what actually ran. The report is stream-json: one JSON object per line after a `# ` header.
# The session id is appended at the end; KIMI_RESUME=<session id> continues that session
# after an interrupted run. Print mode does not exit while a background task is running: it
# waits, then wakes the worker for a new turn, so a long build left in the background
# survives the end of a turn. Foreground commands are capped at 300 s, and background
# tasks have a 600 s default timeout, so the role text tells the worker to set its own.
#
# AGY
#
# `agy` is a fish function wrapping the binary in the local proxy, and the two share one
# name, so name preference cannot protect the proxy the way it does for cdx/codex: the
# probe accepts only a shell FUNCTION named agy and never the bare binary. The prompt goes
# through an exported AGY_TASK, so it needs no quoting. Model and effort are pinned per run
# (AGY_MODEL, default gemini-4-argon; AGY_EFFORT, default high — Argon offers low, medium
# and high). The run is pinned to agy's `default-cli-project`, the project whose grants the
# preflight reads. Before dispatching, the script checks the developer's agy config under
# AGY_HOME (default ~/.gemini): toolPermission must be proceed-in-sandbox,
# allowNonWorkspaceAccess must not be true, a write_file rule must grant this repository,
# and every allow rule must be read_file(...) or write_file(<path inside this
# repository>). It refuses anything else, since a command(...) allow can run a command
# outside the sandbox (agy folded unsandboxed(...) into command(...)), and a URL allow
# opens the network. The report is stream-json after a `# ` header; the conversation id is
# appended at the end, and AGY_RESUME=<conversation id> continues it (`--conversation`).
# Long commands move to the background on their own. Once the agent ends its turn, print
# mode waits at most 30 minutes for them, so the role text tells the worker to keep
# polling instead.
#
# GRK
#
# `grk` is a fish function wrapping the `grok` binary in the local proxy. Only a shell
# FUNCTION named grk is accepted. Model and effort are pinned per run: GRK_MODEL defaults
# to grok-4.7, and GRK_EFFORT defaults to xhigh, the highest grok-4.7 offers (low, medium,
# high, xhigh). Before dispatching, the script reads the `rum-worker` profile in
# sandbox.toml under GROK_HOME (default ~/.grok). The profile must extend strict, restrict
# the network, add no read_write path, and deny this repository's .git.
# requirements.toml and managed_config.toml must not pin a sandbox, since a pin outranks
# --sandbox, and the repo must have no project .grok directory.
# The script then runs a one-word census session under the same flags. It refuses unless
# that session's init event lists exactly the allowed tools and no MCP server, and unless
# the profile grok logged to sessions/sandbox-events.jsonl for it is rum-worker: enforced,
# network-restricted, writing only to the expected paths, with .git denied. The census
# log is <log>.census.
# The report is Anthropic-messages stream JSON (init, assistant turns with their text and
# tool calls, tool results, result) after a `# ` header. The session id is chosen up front
# (-s) and written to the header, so even a run cut before its result event can be
# resumed with GRK_RESUME=<session id> (`--resume`; the session keeps its sandbox profile).
# After its turn ends, a headless run waits --background-wait-timeout for background work
# and then kills it. The script sets that wait to 3900 s, above the brief's 3600-s
# workspace bound, and the role text tells the worker to wait for long commands rather
# than end its turn.
#
# MEMORY SCOPE
#
# Every worker runs inside its own memory-capped systemd user scope
# (scripts/memory-scope.sh): MemoryMax=BRIEF_MEMORY_MAX (default 16G), no swap,
# OOMPolicy=continue. A command that hits the cap is OOM-killed alone (exit 137) and the
# worker carries on; nothing outside the scope is touched, in particular not the
# director's terminal cgroup the worker would otherwise share. The worker cannot make
# scopes of its own: the agy and grk sandboxes hide the user bus. Inside, the worker reads
# its scope's memory.peak and memory.events under /sys/fs/cgroup to see an OOM kill. The
# log ends with a `# memory-scope:` line giving the scope's peak and OOM counts.
#
# DRY_RUN=1 prints the resolved brief/worker/log and exits WITHOUT dispatching (for testing).
set -euo pipefail

usage() {
  echo "usage: scripts/dispatch-brief.sh <brief-id>   (e.g. 081, brief-081)" >&2
  echo "       BRIEF_WORKER=codex|cgl|kimi|agy|grk   (default codex)" >&2
  echo "       BRIEF_SHELL=<shell>      shell that defines the worker command" >&2
  echo "       CDX_BIN / CGL_BIN        worker command name" >&2
  echo "       CDX_FORK=<session id>    codex only: fork that earlier session" >&2
  echo "       KIMI_BIN=<path>          kimi only: the kimi binary (default: PATH lookup)" >&2
  echo "       KIMI_EFFORT=<effort>     kimi only: reasoning effort (default max)" >&2
  echo "       KIMI_RESUME=<session id> kimi only: continue that earlier session" >&2
  echo "       AGY_MODEL=<model>        agy only: model (default gemini-4-argon)" >&2
  echo "       AGY_EFFORT=<effort>      agy only: reasoning effort (default high)" >&2
  echo "       AGY_RESUME=<conv id>     agy only: continue that earlier conversation" >&2
  echo "       AGY_HOME=<dir>           agy only: agy config root (default ~/.gemini)" >&2
  echo "       GRK_MODEL=<model>        grk only: model (default grok-4.7)" >&2
  echo "       GRK_EFFORT=<effort>      grk only: reasoning effort (default xhigh)" >&2
  echo "       GRK_RESUME=<session id>  grk only: continue that earlier session" >&2
  echo "       GROK_HOME=<dir>          grk only: grok config root (default ~/.grok)" >&2
  echo "       BRIEF_MEMORY_MAX=<size>  the worker scope's MemoryMax (default 16G)" >&2
  exit 2
}
[[ $# -eq 1 ]] || usage

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
id="${1#brief-}"   # strip a leading "brief-" if present

worker="${BRIEF_WORKER:-codex}"
case "$worker" in
  codex|cdx) worker=codex; log_tag=cdx ;;
  cgl|glm)   worker=cgl;   log_tag=cgl ;;
  kimi)      worker=kimi;  log_tag=kimi ;;
  agy)       worker=agy;   log_tag=agy ;;
  grk|grok)  worker=grk;   log_tag=grk ;;
  *) echo "error: unknown BRIEF_WORKER '$worker' (expected codex, cgl, kimi, agy or grk)" >&2; exit 2 ;;
esac

shopt -s nullglob
matches=("$repo_root"/docs/dev/codex/brief-"$id"*.md)
[[ ${#matches[@]} -eq 0 ]] && matches=("$repo_root"/docs/dev/codex/*"$id"*.md)
if [[ ${#matches[@]} -eq 0 ]]; then
  echo "error: no brief matching '$1' under docs/dev/codex/" >&2; exit 1
fi
if [[ ${#matches[@]} -gt 1 ]]; then
  echo "error: ambiguous '$1' matches:" >&2; printf '  %s\n' "${matches[@]}" >&2; exit 1
fi

brief="${matches[0]}"
rel="docs/dev/codex/$(basename "$brief")"
num=$(basename "$brief" | sed -E 's/^brief-([0-9]+).*/\1/')
mkdir -p "$repo_root/.temp/logs"
log="$repo_root/.temp/logs/${log_tag}_brief_${num}.log"
task="Do not run git in any form, not even git status: the director owns git. Read $rel first and execute it exactly. Follow its Hard rules. Standing inventories: when an edit changes source they cover, replay them before the first acceptance suite, and quote the replay verdict lines in your report. A suite red caused only by a stale fingerprint, certificate or position cell is an ordering artifact, not a regression: report it as such, and never weaken or hand-edit an inventory."

echo "brief:   $rel"
echo "worker:  $worker"
echo "log:     $log"

if [[ -n "$(git -C "$repo_root" status --porcelain)" ]]; then
  echo "WARNING: working tree is not clean — the worker's changes will mix with existing ones" >&2
fi

# The worker CLIs are interactive-shell functions in the developer's own config,
# not plain binaries: `cdx` runs `codex` through a local proxy, and `cgl`/`glm`
# run `claude` against a GLM backend with an isolated config dir. Two things
# therefore vary by host — which shell holds the definition (zsh on the macOS
# box this workflow grew up on, fish here) and what the function is called — and
# the wrappers are NOT interchangeable with the bare binary, since substituting
# `codex` for `cdx` silently drops the proxy. So probe for a real definition and
# fail loudly rather than fall back to something that merely has the right name.
#
# Name preference dominates shell preference: every shell is tried for `cdx`
# before any shell is offered `codex`.
# An explicit override is authoritative, not a preference: it is the only
# candidate, so a wrong one fails loudly instead of being quietly replaced by
# the probe's next guess.
if [[ "$worker" == codex ]]; then
  worker_names=("${CDX_BIN:-}") && [[ -n "${CDX_BIN:-}" ]] || worker_names=(cdx codex)
elif [[ "$worker" == cgl ]]; then
  worker_names=("${CGL_BIN:-}") && [[ -n "${CGL_BIN:-}" ]] || worker_names=(cgl glm)
elif [[ "$worker" == agy ]]; then
  worker_names=(agy)
elif [[ "$worker" == grk ]]; then
  worker_names=(grk)
else
  worker_names=()   # kimi resolves below, without a shell
fi

if [[ -n "${BRIEF_SHELL:-}" ]]; then
  candidate_shells=("$BRIEF_SHELL")
else
  candidate_shells=()
  login_shell=$(getent passwd "$(id -un)" 2>/dev/null | cut -d: -f7)
  [[ -n "$login_shell" ]] && candidate_shells+=("$login_shell")
  [[ -n "${SHELL:-}" ]] && candidate_shells+=("$SHELL")
  candidate_shells+=(fish zsh bash)
fi

# `type` is a builtin in all three shells and reports functions and aliases,
# which `command -v` from here cannot see. `-i` is what loads the rc file that
# defines them.
shell_defines() { # <shell> <name>
  command -v "$1" >/dev/null 2>&1 || return 1
  "$1" -ic "type $2 >/dev/null 2>&1" </dev/null >/dev/null 2>&1
}

# agy's wrapper and its binary share one name, so `type` would accept the bare
# binary in any shell that lacks the function — and the bare binary drops the
# proxy. Only a shell FUNCTION of that name is accepted.
shell_defines_function() { # <shell> <name>
  command -v "$1" >/dev/null 2>&1 || return 1
  local kind
  case "$(basename "$1")" in
    fish) "$1" -ic "functions -q $2" </dev/null >/dev/null 2>&1 ;;
    bash) kind=$("$1" -ic "type -t $2" </dev/null 2>/dev/null) || return 1
          [[ "$kind" == function ]] ;;
    zsh)  kind=$("$1" -ic "whence -w $2" </dev/null 2>/dev/null) || return 1
          [[ "$kind" == *": function" ]] ;;
    *)    return 1 ;;
  esac
}

# The git fence for workers with no git deny of their own (kimi, agy). PATH is
# set here, in bash, which also clears bash's command hash, so the check below
# sees a fresh lookup.
install_git_fence() {
  fence_dir="$repo_root/.temp/worker-fence/bin"
  git_attempts="$log.git-attempts"
  mkdir -p "$fence_dir"
  cat > "$fence_dir/git" <<'EOF'
#!/bin/sh
# Written by scripts/dispatch-brief.sh: a dispatched worker must not run git.
printf '%s\t%s\tgit %s\n' "$(date -Is)" "$(ps -o args= -p "$PPID" 2>/dev/null)" "$*" \
  >> "${WORKER_GIT_FENCE_LOG:-/dev/null}"
echo "git: refused - the director owns git; a dispatched worker must not run it" >&2
exit 1
EOF
  chmod +x "$fence_dir/git"
  export WORKER_GIT_FENCE_LOG="$git_attempts"
  export PATH="$fence_dir:$PATH"
  if [[ "$(command -v git)" != "$fence_dir/git" ]]; then
    echo "error: the git fence is not first on PATH" >&2
    exit 1
  fi
  echo "git-fence: $fence_dir/git (attempts -> $git_attempts)"
}

# The dispatch shell's rc file could put something ahead of the shim.
check_git_fence_in_dispatch_shell() {
  local seen_git
  seen_git=$("$dispatch_shell" -ic 'command -v git' </dev/null 2>/dev/null | tail -n 1 || true)
  if [[ "$seen_git" != "$fence_dir/git" ]]; then
    echo "error: in $dispatch_shell the git fence is not first on PATH (got '$seen_git')" >&2
    exit 1
  fi
}

dispatch_shell=""
worker_cmd=""
if [[ "$worker" == kimi ]]; then
  # A plain binary needs no rc file, but a wrapper must not be bypassed: if
  # the login shell defines `kimi` as a function, refuse rather than run the
  # bare binary under its name (the `codex`-for-`cdx` trap above).
  if command -v fish >/dev/null 2>&1 \
      && fish -ic 'functions -q kimi' </dev/null >/dev/null 2>&1; then
    echo "error: fish defines kimi as a function; this script only runs the plain binary" >&2
    exit 1
  fi
  worker_cmd="${KIMI_BIN:-$(command -v kimi || true)}"
  if [[ -z "$worker_cmd" || ! -x "$worker_cmd" ]]; then
    echo "error: no kimi binary found (set KIMI_BIN)" >&2
    exit 1
  fi
  dispatch_shell=direct
fi
probe=shell_defines
[[ "$worker" == agy || "$worker" == grk ]] && probe=shell_defines_function
for candidate_name in "${worker_names[@]}"; do
  for candidate_shell in "${candidate_shells[@]}"; do
    if "$probe" "$candidate_shell" "$candidate_name"; then
      dispatch_shell="$candidate_shell"
      worker_cmd="$candidate_name"
      break 2
    fi
  done
done

if [[ -z "$worker_cmd" ]]; then
  echo "error: no shell defines any of: ${worker_names[*]}" >&2
  echo "       shells tried: ${candidate_shells[*]}" >&2
  echo "hint:  the dispatch runs the worker through an interactive shell so that" >&2
  echo "       shell functions and aliases resolve; set BRIEF_SHELL to name the" >&2
  echo "       shell, or CDX_BIN/CGL_BIN to name the command." >&2
  exit 1
fi

echo "command: $worker_cmd ($dispatch_shell)"

if [[ "$worker" == codex ]]; then
  sandbox="${CDX_SANDBOX:-workspace-write}"
  echo "sandbox: $sandbox"
  # codex logs through `tracing` and honours RUST_LOG (the binary carries both
  # RUST_LOG and tracing_subscriber's EnvFilter). At its default only ERROR is
  # emitted: across fourteen dispatch logs and ~1.5M captured lines the only
  # tracing output at all was 25 ERROR lines from two targets. That is why the
  # three silent mid-run terminations left nothing to read - a transport notice
  # such as "Falling back from WebSockets to HTTPS transport" is WARN or INFO
  # and could never have appeared. Capture WARN globally plus INFO from
  # codex_core, which is where the client/transport lives.
  rust_log="${BRIEF_RUST_LOG:-warn,codex_core=info}"
  echo "rust-log: $rust_log"
  # Exported here, in bash, rather than prefixed onto the inner command. The
  # worker is a SHELL FUNCTION, so `env VAR=v cdx ...` cannot work — env execs a
  # binary and would fail to find it — and a bare `VAR=v cmd` prefix is not
  # valid in fish, which is the dispatch shell on this host. Exporting lets the
  # interactive shell inherit it whatever its syntax.
  export RUST_LOG="$rust_log"
  if [[ -n "${CDX_FORK:-}" ]]; then
    # CDX_FORK=<session id> forks an earlier Codex session (its id is the
    # `session id:` line of that brief's log) so the worker keeps what it read
    # and concluded there — e.g. a design round built on the same worker's
    # review. Forking leaves the earlier session untouched. `exec fork` takes
    # no -C/-s flags, so the directory comes from the cd and the sandbox from
    # a config override; the log's `sandbox:` header line confirms it applied.
    [[ "$CDX_FORK" =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] \
      || { echo "error: CDX_FORK must be a session UUID, got '$CDX_FORK'" >&2; exit 2; }
    echo "fork-of: $CDX_FORK"
    task="This session is forked from an earlier session of yours; that context is background and the brief governs. $task"
    cmd="cd '$repo_root' && $worker_cmd exec fork -c sandbox_mode=$sandbox '$CDX_FORK' '$task'"
  else
    cmd="$worker_cmd exec -C '$repo_root' -s $sandbox '$task'"
  fi
elif [[ "$worker" == kimi ]]; then
  effort="${KIMI_EFFORT:-max}"
  echo "effort:  $effort (KIMI_MODEL_THINKING_EFFORT)"
  echo "permission-mode: auto, forced by -p (no write fence — see header)"
  export KIMI_MODEL_THINKING_EFFORT="$effort"
  install_git_fence
  # NOTE: keep this text free of single quotes, like cgl's role below.
  role="You are the implementation worker for this brief, not the project director. \
CLAUDE.md belongs to the director and is NOT your instruction file. \
Follow AGENTS.md and the brief. Do not commit, branch, or push; leave changes in the \
working tree. This run is non-interactive: nobody answers questions, and the run ends \
when you end a turn with no background task running, so do not end your turn until the \
brief is finished or one of its stop conditions has fired. A foreground command is capped \
at 300 seconds: run longer builds and tests as background tasks, and give each one an \
explicit timeout long enough for it, because the 600-second background default kills a \
full test run. Your own tool timeout must exceed any bound the command sets itself, such as \
the run-bounded.py timeout: a 300-second tool timeout kills a command bounded at 3600. \
Wait for each one to finish before you go on. \
Write the report file the brief's Report format section names, run scripts/report.py check on it until it prints no PROBLEM, and make your final message the output of scripts/report.py summary."
  kimi_task="$role $task"
  kimi_args=()
  if [[ -n "${KIMI_RESUME:-}" ]]; then
    [[ "$KIMI_RESUME" =~ ^session_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] \
      || { echo "error: KIMI_RESUME must be a kimi session id, got '$KIMI_RESUME'" >&2; exit 2; }
    echo "resume-of: $KIMI_RESUME"
    kimi_task="This continues your own earlier run of this brief, which was interrupted or \
answered by a new director amendment at the end of the brief. Re-read the brief, the working \
tree and any checkpoint the brief asks for before you go on; the brief governs. $kimi_task"
    kimi_args=(-S "$KIMI_RESUME")
  fi
  kimi_args+=(-p "$kimi_task" --output-format stream-json)
  cmd="cd $(printf '%q' "$repo_root") && $(printf '%q ' "$worker_cmd" "${kimi_args[@]}")"
elif [[ "$worker" == agy ]]; then
  model="${AGY_MODEL:-gemini-4-argon}"
  effort="${AGY_EFFORT:-high}"
  [[ "$model" =~ ^[A-Za-z0-9._-]+$ && "$effort" =~ ^[a-z]+$ ]] \
    || { echo "error: AGY_MODEL/AGY_EFFORT must be plain names, got '$model'/'$effort'" >&2; exit 2; }
  echo "model:   $model (effort $effort)"
  # The fence is the developer's agy config, so check it rather than trust it.
  agy_home="${AGY_HOME:-$HOME/.gemini}"
  agy_fence=$(python3 - "$agy_home" "$repo_root" <<'PY'
import json, os, sys

home, repo = sys.argv[1], os.path.realpath(sys.argv[2])

def load(*parts):
    path = os.path.join(home, *parts)
    try:
        with open(path) as f:
            return json.load(f)
    except FileNotFoundError:
        return {}

settings = load("antigravity-cli", "settings.json")
project = load("config", "projects", "default-cli-project.json")
rules = [("settings.json", r) for r in (settings.get("permissions") or {}).get("allow") or []]
grants = ((project.get("permissionGrants") or {}).get("permissionGrants") or {}).get("allow") or []
rules += [("default-cli-project", r) for r in grants]

def inside_repo(target):
    path = os.path.realpath(target)
    return path == repo or path.startswith(repo + os.sep)

errors, repo_granted = [], False
if settings.get("toolPermission") != "proceed-in-sandbox":
    errors.append("toolPermission is %r, not proceed-in-sandbox: the first command would be "
                  "denied and end the run" % settings.get("toolPermission"))
if settings.get("allowNonWorkspaceAccess") is True:
    errors.append("allowNonWorkspaceAccess is true: the file tool could write outside the repository")
for origin, rule in rules:
    action, _, rest = rule.partition("(")
    target = rest[:-1] if rest.endswith(")") else None
    if action == "read_file" and target:
        continue
    if action == "write_file" and target and not target.startswith("regex:") \
            and os.path.isabs(target) and inside_repo(target):
        repo_granted = repo_granted or os.path.realpath(target) == repo
        continue
    errors.append("%s allows %s: only read_file(...) and write_file(<path in %s>) keep the "
                  "fence" % (origin, rule, repo))
if not repo_granted:
    errors.append("no allow rule write_file(%s): the repository is read-only in the sandbox" % repo)
if errors:
    for e in errors:
        print("error: agy fence: " + e, file=sys.stderr)
    sys.exit(1)
print("sandbox, proceed-in-sandbox, %d allow rule(s), write_file(%s)" % (len(rules), repo))
PY
) || exit 1
  echo "agy-fence: $agy_fence"
  install_git_fence
  check_git_fence_in_dispatch_shell
  agy_version=$("$dispatch_shell" -ic "$worker_cmd changelog" </dev/null 2>/dev/null \
    | sed -n '1s/:$//p' || true)
  role="You are the implementation worker for this brief, not the project director. \
CLAUDE.md belongs to the director and is NOT your instruction file. \
Follow AGENTS.md and the brief. Do not commit, branch, or push; leave changes in the \
working tree. This run is non-interactive and sandboxed: nobody answers questions or \
approves anything, and any action that needs approval is denied and ENDS THE RUN at once. \
So never ask to run a command outside the sandbox, never write with the file tool outside \
this repository (keep scratch under its .temp directory), and do not use the browser, web \
search or ask_question; there is no network. Do the work yourself, not through subagents, \
so that every step is in this run. Long commands move to the background: do not end your \
turn while one is running, but poll it with command_status until it finishes, because once \
you end your turn the CLI waits at most 30 minutes for background tasks and a full test \
run takes longer. Do not end your turn until the brief is finished or one of its stop \
conditions has fired. Write the report file the brief's Report format section names, run scripts/report.py check on it until it prints no PROBLEM, and make your final message the output of scripts/report.py summary."
  agy_task="$role $task"
  agy_args="--model $model --effort $effort --output-format stream-json --print-timeout 0 \
--sandbox --project default-cli-project"
  if [[ -n "${AGY_RESUME:-}" ]]; then
    [[ "$AGY_RESUME" =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] \
      || { echo "error: AGY_RESUME must be a conversation UUID, got '$AGY_RESUME'" >&2; exit 2; }
    echo "resume-of: $AGY_RESUME"
    agy_task="This continues your own earlier run of this brief, which was interrupted or \
answered by a new director amendment at the end of the brief. Re-read the brief, the working \
tree and any checkpoint the brief asks for before you go on; the brief governs. $agy_task"
    agy_args="$agy_args --conversation $AGY_RESUME"
  fi
  # Through the environment, so the prompt needs no quoting in any shell.
  export AGY_TASK="$agy_task"
  cmd="cd '$repo_root' && $worker_cmd -p \"\$AGY_TASK\" $agy_args"
elif [[ "$worker" == grk ]]; then
  model="${GRK_MODEL:-grok-4.7}"
  effort="${GRK_EFFORT:-xhigh}"
  [[ "$model" =~ ^[A-Za-z0-9._-]+$ && "$effort" =~ ^[a-z]+$ ]] \
    || { echo "error: GRK_MODEL/GRK_EFFORT must be plain names, got '$model'/'$effort'" >&2; exit 2; }
  echo "model:   $model (effort $effort)"
  # The fence is a profile in the developer's grok config, so check it rather
  # than trust it; the census session below then observes what grok applied.
  grok_home="${GROK_HOME:-$HOME/.grok}"
  grk_profile=rum-worker
  grk_fence=$(python3 - "$grok_home" "$repo_root" "$grk_profile" <<'PY'
import os, sys, tomllib

home, repo, name = sys.argv[1], os.path.realpath(sys.argv[2]), sys.argv[3]

def load(path):
    try:
        with open(path, "rb") as f:
            return tomllib.load(f)
    except FileNotFoundError:
        return {}

errors = []
profile = (load(os.path.join(home, "sandbox.toml")).get("profiles") or {}).get(name)
if profile is None:
    errors.append("no [profiles.%s] in %s/sandbox.toml" % (name, home))
    profile = {}
if profile.get("extends") != "strict":
    errors.append("the profile extends %r, not strict: reads would reach the whole home"
                  % profile.get("extends"))
if profile.get("restrict_network") is not True:
    errors.append("restrict_network is not true: child processes would have the network")
if profile.get("read_write"):
    errors.append("read_write grants %r: writes must stay in the repository and temp dirs"
                  % profile["read_write"])
# grok anchors a relative deny entry at the workspace and `~/` at the home.
deny = [os.path.realpath(os.path.join(repo, os.path.expanduser(p)))
        for p in profile.get("deny") or []]
git_dir = os.path.join(repo, ".git")
if git_dir not in deny:
    errors.append("deny does not list %s: the worker could write git metadata" % git_dir)
for pinned in ("requirements.toml", "managed_config.toml"):
    if "sandbox" in load(os.path.join(home, pinned)):
        errors.append("%s pins a sandbox, which outranks --sandbox" % pinned)
if os.path.exists(os.path.join(repo, ".grok")):
    errors.append("%s/.grok exists: a project grok config would join the session" % repo)
if errors:
    for e in errors:
        print("error: grk fence: " + e, file=sys.stderr)
    sys.exit(1)
print("profile %s: strict, network restricted, %s denied, read-only %s"
      % (name, git_dir, ", ".join(profile.get("read_only") or []) or "-"))
PY
) || exit 1
  echo "grk-fence: $grk_fence"
  install_git_fence
  check_git_fence_in_dispatch_shell
  grk_version=$("$dispatch_shell" -ic "$worker_cmd --version" </dev/null 2>/dev/null | tail -n 1 || true)
  # grok's own environment switches: no cross-session memory, no self-update
  # mid-run, and none of the Claude Code or Cursor configuration grok loads by
  # default (skills, rules, CLAUDE.md, hooks, MCP servers). ~/.claude.json on
  # this host declares a Wolfram engine, which the testing policy forbids a
  # worker to reach.
  export GROK_MEMORY=0 GROK_DISABLE_AUTOUPDATER=1
  for layer in CLAUDE CURSOR; do
    for part in SKILLS RULES AGENTS MCPS HOOKS; do
      export "GROK_${layer}_${part}_ENABLED=0"
    done
  done
  # The allowlist by tool id; the census checks the names the model is shown.
  # search_tool and use_tool would reach deferred tools behind the allowlist.
  grk_tools=run_terminal_cmd,read_file,search_replace,write,list_dir,grep,todo_write,get_task_output,kill_task,wait_tasks
  grk_shown="get_command_or_subagent_output grep kill_command_or_subagent list_dir read_file \
run_terminal_command search_replace todo_write write"
  grk_fence_args="--output-format streaming-messages-json --cwd '$repo_root' \
--sandbox $grk_profile --always-approve --disable-web-search --no-subagents --no-auto-update \
--tools $grk_tools --disallowed-tools search_tool,use_tool"
  role="You are the implementation worker for this brief, not the project director. \
CLAUDE.md belongs to the director and is NOT your instruction file. \
Follow AGENTS.md and the brief. Do not commit, branch, or push; leave changes in the \
working tree. This run is non-interactive and runs inside a kernel sandbox: nobody answers \
questions, you can write only inside this repository and the temp directories (keep all \
scratch under the .temp directory of this repository), child processes have no network, \
the .git directory is not accessible, and home directories outside the Rust toolchain are \
unreadable. A command the sandbox refuses fails with a permission error: report it, and do \
not work around it. Do the work yourself, without subagents. A shell command that outlives \
its block_until_ms is not killed: it moves to the background and keeps running. \
Run long builds and tests in the background and wait for each one with \
get_command_or_subagent_output and a timeout. A single wait returns after at most one hour, \
so wait again while the command still runs. Give every command a bound of its own, and \
make it longer than the command needs. Never end your turn while a background command \
runs: once your turn ends, the run waits about an hour for background work and then kills \
it. Do not end your turn until the brief is finished or one of its stop conditions has \
fired. Keep the checkpoint and report files the brief asks for current as you go: a run \
cut by a usage limit keeps only what is already on disk. Write the report file the brief's \
Report format section names, run scripts/report.py check on it until it prints no PROBLEM, \
and make your final message the output of scripts/report.py summary."
  grk_task="$role $task"
  if [[ -n "${GRK_RESUME:-}" ]]; then
    [[ "$GRK_RESUME" =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] \
      || { echo "error: GRK_RESUME must be a session UUID, got '$GRK_RESUME'" >&2; exit 2; }
    echo "resume-of: $GRK_RESUME"
    grk_session="$GRK_RESUME"
    grk_session_args="--resume $grk_session"
    grk_task="This continues your own earlier run of this brief, which was interrupted or \
answered by a new director amendment at the end of the brief. Re-read the brief, the working \
tree and any checkpoint the brief asks for before you go on; the brief governs. $grk_task"
  else
    grk_session=$(python3 -c 'import uuid; print(uuid.uuid4())')
    grk_session_args="-s $grk_session"
  fi
  echo "session: $grk_session"
  # Through the environment, so the prompt needs no quoting in any shell.
  export GRK_TASK="$grk_task"
  cmd="cd '$repo_root' && $worker_cmd -p \"\$GRK_TASK\" -m $model --effort $effort \
$grk_fence_args --background-wait-timeout 3900 $grk_session_args"
else
  permission_mode="${CGL_PERMISSION_MODE:-acceptEdits}"
  echo "permission-mode: $permission_mode (no OS-level write fence — see header)"
  # Keep the worker from adopting CLAUDE.md, which is the director's file, not the worker's.
  # NOTE: keep this text free of single quotes — it is embedded in a single-quoted
  # interactive-shell command below, and an apostrophe would terminate that string
  # early (true in zsh, bash and fish alike).
  role="You are the implementation worker for this brief, not the project director. \
CLAUDE.md belongs to the director and is NOT your instruction file. \
Follow AGENTS.md and the brief. Do not commit, branch, or push; leave changes in the \
working tree. Write the report file the brief's Report format section names, run scripts/report.py check on it until it prints no PROBLEM, and make your final message the output of scripts/report.py summary."
  # A single response is capped at CLAUDE_CODE_MAX_OUTPUT_TOKENS; exceeding it
  # kills the run and leaves nothing behind (brief-307 died that way at the
  # 32000 default). The cap belongs to cgl, not to this repo, so it lives in
  # ~/.claude-glm/settings.json; set CGL_MAX_OUTPUT_TOKENS only to override a
  # single dispatch.
  if [[ -n "${CGL_MAX_OUTPUT_TOKENS:-}" ]]; then
    # Exported for the same reason as RUST_LOG above: `cgl` is a shell function
    # and the dispatch shell may be fish, so a prefix assignment would not run.
    export CLAUDE_CODE_MAX_OUTPUT_TOKENS="$CGL_MAX_OUTPUT_TOKENS"
    echo "max-output-tokens: $CGL_MAX_OUTPUT_TOKENS (overriding cgl default)"
  fi
  cmd="cd '$repo_root' && $worker_cmd -p '$task' \
--permission-mode $permission_mode \
--allowedTools Bash Edit Write Read Grep Glob \
--append-system-prompt '$role'"
fi

memory_max="${BRIEF_MEMORY_MAX:-16G}"
scope_unit="brief-$num-$log_tag-$(date +%Y%m%d-%H%M%S)-$$"
scope=("$repo_root/scripts/memory-scope.sh" "$scope_unit" "$memory_max")
echo "memory-scope: $scope_unit MemoryMax=$memory_max"

if [[ "${DRY_RUN:-0}" == "1" ]]; then
  echo "DRY_RUN: not dispatching"
  echo "would run: $cmd"
  exit 0
fi

# Refuse rather than run unbounded: the scope is what keeps an OOM inside the job.
if ! "${scope[@]}" true 2>/dev/null; then
  echo "error: cannot create a memory scope (systemd-run --user); nothing dispatched" >&2
  exit 1
fi

set +e
if [[ "$worker" == kimi ]]; then
  : > "$git_attempts"
  {
    echo "# dispatch: worker=kimi brief=$rel effort=$effort started=$(date -Is)"
    echo "# command: $worker_cmd ($("$worker_cmd" --version 2>/dev/null))"
    echo "# git-fence: $fence_dir/git"
    [[ -n "${KIMI_RESUME:-}" ]] && echo "# resume-of: $KIMI_RESUME"
  } > "$log"
  (cd "$repo_root" && "${scope[@]}" "$worker_cmd" "${kimi_args[@]}") < /dev/null >> "$log" 2>&1
  rc=$?
  # The stream names the session only in its last event, so an interrupted run
  # falls back to the newest session recorded for this directory.
  session=$(grep -o -E '"session_id":"session_[0-9a-f-]+"' "$log" | tail -n 1 | cut -d'"' -f4)
  if [[ -z "$session" ]]; then
    session=$("$worker_cmd" session list --cwd "$repo_root" --limit 1 </dev/null 2>/dev/null \
      | grep -o -E 'session_[0-9a-f]{8}-[0-9a-f-]+' | head -n 1)
  fi
  echo "# kimi-session: ${session:-unknown}" >> "$log"
  echo "kimi-session: ${session:-unknown}"
  echo "git-attempts: $(wc -l < "$git_attempts") (in $git_attempts)"
elif [[ "$worker" == agy ]]; then
  : > "$git_attempts"
  {
    echo "# dispatch: worker=agy brief=$rel model=$model effort=$effort started=$(date -Is)"
    echo "# command: $worker_cmd ($dispatch_shell) agy ${agy_version:-unknown}"
    echo "# agy-fence: $agy_fence"
    echo "# git-fence: $fence_dir/git"
    [[ -n "${AGY_RESUME:-}" ]] && echo "# resume-of: $AGY_RESUME"
  } > "$log"
  "${scope[@]}" "$dispatch_shell" -ic "$cmd" < /dev/null >> "$log" 2>&1
  rc=$?
  # A denied action ends the run with exit 0; only the result event says so.
  summary=$(python3 - "$log" <<'PY'
import json, sys
conv, status, denied = "", "none", []
for line in open(sys.argv[1], errors="replace"):
    try:
        event = json.loads(line)
    except ValueError:
        continue
    if not isinstance(event, dict):
        continue
    if not conv:
        conv = (event.get("init") or {}).get("conversation_id") or ""
    result = event.get("result")
    if event.get("event") == "result" and isinstance(result, dict):
        conv = result.get("conversation_id") or conv
        status = result.get("status") or "unknown"
        denied = [d.get("action", "?") for d in result.get("denied_actions") or []]
print(conv or "unknown", status, ",".join(denied) or "-")
PY
)
  read -r conversation status denied <<< "$summary"
  echo "# agy-conversation: $conversation" >> "$log"
  echo "# agy-result: status=$status denied=$denied" >> "$log"
  echo "agy-conversation: $conversation"
  echo "agy-result: status=$status denied=$denied"
  echo "git-attempts: $(wc -l < "$git_attempts") (in $git_attempts)"
  if [[ $rc -eq 0 && "$denied" != "-" ]]; then
    echo "WARNING: the run ended on a denied action ($denied); resume with AGY_RESUME=$conversation" >&2
    rc=3
  fi
elif [[ "$worker" == grk ]]; then
  # The census: one cheap session under the real fence flags, read back before
  # the brief runs. The init event names the toolset the model is shown, and
  # grok logs the kernel profile it applied.
  census_log="$log.census"
  census_session=$(python3 -c 'import uuid; print(uuid.uuid4())')
  census_started=$(date -u +%Y-%m-%dT%H:%M:%S)
  "$dispatch_shell" -ic "cd '$repo_root' && $worker_cmd -p 'Reply with the single word OK. \
Use no tool.' -m $model --effort low $grk_fence_args -s $census_session" \
    < /dev/null > "$census_log" 2>&1
  census_rc=$?
  grk_census=$(python3 - "$census_log" "$grok_home" "$census_started" "$repo_root" \
      "$grk_profile" "$grk_shown" "$census_rc" <<'PY'
import json, os, sys

log, home, started, repo, name, shown, rc = sys.argv[1:]
repo = os.path.realpath(repo)
init = result = None
for line in open(log, errors="replace"):
    try:
        event = json.loads(line)
    except ValueError:
        continue
    if not isinstance(event, dict):
        continue
    if event.get("type") == "system" and event.get("subtype") == "init":
        init = event
    elif event.get("type") == "result":
        result = event
errors = []
if rc != "0" or result is None or result.get("is_error"):
    errors.append("the census session did not complete (exit %s)" % rc)
if init is None:
    errors.append("the census session logged no init event")
else:
    tools = sorted(init.get("tools") or [])
    if tools != sorted(shown.split()):
        errors.append("the model is shown %s, expected %s" % (tools, sorted(shown.split())))
    if init.get("mcp_servers"):
        errors.append("MCP servers are loaded: %r" % init["mcp_servers"])
    if init.get("permissionMode") != "bypassPermissions":
        errors.append("permission mode %r, expected bypassPermissions" % init.get("permissionMode"))
applied = None
try:
    for line in open(os.path.join(home, "sessions", "sandbox-events.jsonl"), errors="replace"):
        try:
            event = json.loads(line)
        except ValueError:
            continue
        if isinstance(event, dict) and event.get("event_type") == "ProfileApplied" \
                and event.get("timestamp", "") >= started and event.get("workspace") == repo:
            applied = event
except FileNotFoundError:
    pass
writable = {repo, os.path.join(home, "sessions"), "/tmp", "/var/tmp"}
git_dir = os.path.join(repo, ".git")
if applied is None:
    errors.append("grok logged no ProfileApplied event for the census session")
else:
    if applied.get("profile") != name or applied.get("enforced") is not True:
        errors.append("applied profile %r, enforced=%r" % (applied.get("profile"), applied.get("enforced")))
    if applied.get("restrict_network") is not True:
        errors.append("the applied profile leaves child processes the network")
    extra = sorted(set(applied.get("read_write_paths") or []) - writable)
    if extra:
        errors.append("the applied profile also writes %s" % extra)
    if git_dir not in (applied.get("deny_paths") or []):
        errors.append("the applied profile does not deny %s" % git_dir)
if errors:
    for e in errors:
        print("error: grk census: " + e, file=sys.stderr)
    sys.exit(1)
print("%d tools, no MCP, bypassPermissions; %s enforced, writes %s, %s denied"
      % (len(init["tools"]), name, " ".join(sorted(applied["read_write_paths"])), git_dir))
PY
)
  census_ok=$?
  if [[ $census_ok -ne 0 ]]; then
    echo "error: grk census failed; nothing dispatched (see $census_log)" >&2
    exit 1
  fi
  echo "grk-census: $grk_census"
  : > "$git_attempts"
  {
    echo "# dispatch: worker=grk brief=$rel model=$model effort=$effort started=$(date -Is)"
    echo "# command: $worker_cmd ($dispatch_shell) ${grk_version:-unknown}"
    echo "# grk-fence: $grk_fence"
    echo "# grk-census: $grk_census"
    echo "# git-fence: $fence_dir/git"
    echo "# grk-session: $grk_session"
    [[ -n "${GRK_RESUME:-}" ]] && echo "# resume-of: $GRK_RESUME"
  } > "$log"
  "${scope[@]}" "$dispatch_shell" -ic "$cmd" < /dev/null >> "$log" 2>&1
  rc=$?
  # A refused tool call does not end a headless run; only stderr names it.
  summary=$(python3 - "$log" <<'PY'
import json, sys
status, turns, cost, refused = "none", "-", "-", 0
for line in open(sys.argv[1], errors="replace"):
    if line.startswith("permission required:"):
        refused += 1
    try:
        event = json.loads(line)
    except ValueError:
        continue
    if isinstance(event, dict) and event.get("type") == "result":
        status = event.get("subtype") or "unknown"
        if event.get("is_error"):
            status += "/error"
        turns = event.get("num_turns", "-")
        cost = event.get("total_cost_usd", "-")
print(status, turns, cost, refused)
PY
)
  read -r status turns cost refused <<< "$summary"
  echo "# grk-result: status=$status turns=$turns cost_usd=$cost refused=$refused" >> "$log"
  echo "grk-session: $grk_session"
  echo "grk-result: status=$status turns=$turns cost_usd=$cost refused=$refused"
  echo "git-attempts: $(wc -l < "$git_attempts") (in $git_attempts)"
  if [[ $rc -eq 0 && "$refused" != 0 ]]; then
    echo "WARNING: $refused tool call(s) were refused and skipped; see the log" >&2
    rc=3
  fi
  if [[ $rc -ne 0 || "$status" != success ]]; then
    echo "hint: resume with GRK_RESUME=$grk_session BRIEF_WORKER=grk scripts/dispatch-brief.sh $num" >&2
  fi
else
  "${scope[@]}" "$dispatch_shell" -ic "$cmd" < /dev/null > "$log" 2>&1
  rc=$?
fi
set -e
grep -a '^# memory-scope:' "$log" | tail -n 1 || true
echo "$worker exit=$rc ; report in $log"
exit $rc
