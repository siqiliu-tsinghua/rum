# rum-director (a Claude Code mod)

The director's session discipline for this repository, as Claude Code hooks.
Outside the rum repository every hook passes through.

- **Dispatch guard.** A `scripts/dispatch-brief.sh` (or `cdx exec`/`codex exec`)
  call must be its own Bash call with `run_in_background: true`: not chained,
  piped or `&`-detached. `DRY_RUN=1` is exempt. This replaces the old
  `.claude/hooks/guard-dispatch.py`, which had not been wired since the move
  to Linux.
- **Commit guard**, before `git commit` runs:
  - staging and committing are separate Bash calls (no `git add … && git
    commit`, no `commit -a`), so the guard reads the final staged set;
  - `.temp/director/state.md`, the director's handoff, has been rewritten
    since the last commit;
  - a commit that stages `crates/` also stages `TODO.md` and a brief file with
    a `Director gate` section.

  A source commit that lands with no memory file changed since the previous
  commit raises a toast, a reminder rather than a refusal.
- **Compaction after every commit.** When the turn that committed ends with
  the context at 200,000 tokens or more, the mod compacts
  (`$.session.compact`) with handoff instructions and appends `state.md` to
  the compacted conversation. If a new turn has already started, it retries
  at that turn's end.
- **Every other compaction** (manual, automatic, precomputed) gets the same
  instructions; the manual and automatic ones also get `state.md` appended.
- The status line shows the context size after each turn.

Loading: `CLAUDE_CODE_PLUGIN_DIRS` in the `env` block of `~/.claude/settings.json`
names this folder (a project's settings cannot set it).

Checks: `claude plugin validate tools/claude-mod/rum-director` and
`claude plugin test tools/claude-mod/rum-director`.
