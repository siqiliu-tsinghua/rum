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
# DRY_RUN=1 prints the resolved brief/worker/log and exits WITHOUT dispatching (for testing).
set -euo pipefail

usage() {
  echo "usage: scripts/dispatch-brief.sh <brief-id>   (e.g. 081, brief-081)" >&2
  echo "       BRIEF_WORKER=codex|cgl|kimi   (default codex)" >&2
  echo "       BRIEF_SHELL=<shell>      shell that defines the worker command" >&2
  echo "       CDX_BIN / CGL_BIN        worker command name" >&2
  echo "       CDX_FORK=<session id>    codex only: fork that earlier session" >&2
  echo "       KIMI_BIN=<path>          kimi only: the kimi binary (default: PATH lookup)" >&2
  echo "       KIMI_EFFORT=<effort>     kimi only: reasoning effort (default max)" >&2
  echo "       KIMI_RESUME=<session id> kimi only: continue that earlier session" >&2
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
  *) echo "error: unknown BRIEF_WORKER '$worker' (expected codex, cgl or kimi)" >&2; exit 2 ;;
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
task="Do not run git in any form, not even git status: the director owns git. Read $rel first and execute it exactly. Follow its Hard rules."

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
for candidate_name in "${worker_names[@]}"; do
  for candidate_shell in "${candidate_shells[@]}"; do
    if shell_defines "$candidate_shell" "$candidate_name"; then
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
  # The git fence. PATH is set here, in bash, which also clears bash's
  # command hash, so the check below sees a fresh lookup.
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
  # NOTE: keep this text free of single quotes, like cgl's role below.
  role="You are the implementation worker for this brief, not the project director. \
CLAUDE.md belongs to the director and is NOT your instruction file. \
Follow AGENTS.md and the brief. Do not commit, branch, or push; leave changes in the \
working tree. This run is non-interactive: nobody answers questions, and the run ends \
when you end a turn with no background task running, so do not end your turn until the \
brief is finished or one of its stop conditions has fired. A foreground command is capped \
at 300 seconds: run longer builds and tests as background tasks, and give each one an \
explicit timeout long enough for it, because the 600-second background default kills a \
full test run. Wait for each one to finish before you go on. \
Report with verbatim evidence in your final message."
  kimi_task="$role $task"
  kimi_args=()
  if [[ -n "${KIMI_RESUME:-}" ]]; then
    [[ "$KIMI_RESUME" =~ ^session_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] \
      || { echo "error: KIMI_RESUME must be a kimi session id, got '$KIMI_RESUME'" >&2; exit 2; }
    echo "resume-of: $KIMI_RESUME"
    kimi_task="This continues your own interrupted run of this brief. Re-read the working \
tree and any checkpoint the brief asks for before you go on; the brief governs. $kimi_task"
    kimi_args=(-S "$KIMI_RESUME")
  fi
  kimi_args+=(-p "$kimi_task" --output-format stream-json)
  cmd="cd $(printf '%q' "$repo_root") && $(printf '%q ' "$worker_cmd" "${kimi_args[@]}")"
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
working tree. Report with verbatim evidence."
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

if [[ "${DRY_RUN:-0}" == "1" ]]; then
  echo "DRY_RUN: not dispatching"
  echo "would run: $cmd"
  exit 0
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
  (cd "$repo_root" && "$worker_cmd" "${kimi_args[@]}") < /dev/null >> "$log" 2>&1
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
else
  "$dispatch_shell" -ic "$cmd" < /dev/null > "$log" 2>&1
  rc=$?
fi
set -e
echo "$worker exit=$rc ; report in $log"
exit $rc
