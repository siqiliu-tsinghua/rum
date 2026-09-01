# brief-401 — diagnose: does abandoning a suspension leave dynamic scope installed?

**Scope: NONE — diagnosis and plan only, no source change.** Reading, grepping
and throwaway probes are in scope; revert any temporary instrumentation before
reporting and name it.

**Write your findings before running any verification commands.**

## Where this came from

brief-400 stopped on a blocker rather than exceeding its scope, which was the
right call. While writing a deterministic test for cancellation across input
suspension, it found that cancelling a suspended evaluation cleared
`pending_suspension` and dropped the saved continuation frames **without
driving their error cleanup**, so a `Block`'s dynamic binding stayed installed
after the request ended:

```
left:  "List[8, inside]"
right: "List[8, outside]"
```

Its diagnosis: correct abandonment must resume the saved frame driver with
`Err(EvalError::Cancelled)` so every continuation frame unwinds its own scope,
while no host input is converted or delivered. The driver and the private frame
queue live in `crates/rum-runtime/src/runtime/suspension/core.rs`, outside that
brief's scope, so it stopped and asked for an amendment.

brief-400's work is stashed; you are working against the committed tree.

## The question this brief must answer first

I told the user this looks like a **pre-existing** defect rather than one
brief-400 introduced, and split it out on that basis. **That is a hypothesis,
not an established fact, and your first job is to settle it.**

`clear_pending_suspension_state` is committed code
(`crates/rum-runtime/src/runtime/suspension/core.rs:159-161`), and committed
callers drop a pending suspension on paths that do not obviously unwind frames:

- `crates/rum-runtime/src/runtime/builtins_input.rs:233`
- `crates/rum-runtime/src/runtime/suspension/core.rs:165`
- `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:115` and `:146`, both
  on `Err(EvalError::InputRequired)`
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:160-161`, in the
  request checkpoint rollback (`if !snapshot.had_pending_suspension { … = None }`)

So:

1. **Is the leak reachable on the committed tree, with no brief-400 code?**
   Build a probe from the ordinary public surface — a `Block` (and separately a
   `Module` and a `With`) whose body suspends on `InputString`, then reach each
   of the abandonment paths above that you can actually reach, and observe
   whether the dynamic binding survives the request. Report each path as
   reachable-and-leaking, reachable-and-clean, or unreachable-from-here, with
   `path:line` and the probe.
2. If it is **not** reachable today, say so plainly. That makes it a defect in
   brief-400's new cancellation branch, the split was wrong, and the repair
   belongs back inside brief-400. I would rather learn that now than build a
   line on a bad premise.
3. Note the standing-debt entry this brushes against: `TODO.md` records that
   "`Monitor` refresh and `ValueQ` rollback remain best-effort unless their
   documented contract is deliberately promoted." Say whether what you find in
   `monitor.rs` is that documented best-effort limitation or something worse,
   and do not silently promote a contract.

## What else to establish

4. **What exactly leaks?** Dynamic scope (`Block`) is the observed case. Check
   the other state a dropped frame stack can own: `Module`/`With` gensym
   bindings and their reclamation, `$Context` and context-path changes, open
   `Reap` buckets, `Monitor` refresh state, attribute or `Protected` changes
   made inside the abandoned body, and anything else the frame inventory says a
   frame can hold. The suspension-roots line built
   `SuspensionFrame: FrameRootTrace` and a repository invariant test listing
   every handwritten traversal — use that inventory as the checklist rather
   than guessing, and say which frames own unwind-relevant state.
5. **Is the same shape reachable through cancellation as it exists today**,
   without the accepted-request work? The host has `cancel_handle()` and the
   runtime has an interrupt flag. If a cancel that lands during a suspension can
   reach any of the abandonment paths, that is a live defect on the current
   published surface, and it should be described as one.
6. **What is the right repair, and where does it live?** brief-400 proposed a
   runtime-private entrypoint in `suspension/core.rs` that drives the saved
   frames with `Err(EvalError::Cancelled)`. Confirm or improve that. State what
   the entrypoint's contract is, what it must guarantee about not running
   ordinary continuation effects while unwinding, how re-entrancy and nested
   suspension are handled, and how it interacts with the checkpoint rollback in
   `session_helpers.rs`. **Prefer a shape the type system enforces** — a frame
   that can be abandoned only through the unwinding entrypoint beats a rule that
   callers must remember.

## Deliverable

- The reachability verdict for each committed abandonment path, with probes.
- The leaked-state inventory from question 4.
- The repair design, with the entrypoint contract.
- A step-by-step plan, each step independently gateable, naming files, gate
  tests, and what must not break. Say which step brief-400 needs in order to
  resume, since its scope amendment depends on this.
- **Drafts of the follow-up implementation briefs** in `TEMPLATE.md` shape.
- Risks, open questions, and corrections to the background above.

## Verification and cost

No source change, so no test gate. If your plan's repair would touch the
evaluation hot path, say which part and why; the abandonment path itself is a
cold error path and a repair confined to it needs **no timing A/B** — say so
explicitly if that is the case, rather than performing the ceremony.

Where you do need to show a cost claim, prefer the env-gated allocation
counters over timings: they are load-immune and answer "did the ordinary path
gain work" deterministically. This host's same-binary microbench spread was
measured at 0.05%-18.75% per label even pinned to P-cores, so a timing A/B here
could not resolve anything a counter would not answer better.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do not `git stash`,
  `git checkout`, or otherwise disturb the working tree — **there is a stash
  holding brief-400's work in progress; leave it alone.**
- Preserve the pre-existing worktree changes (five modified files under
  `scripts/` and `.claude/`, plus untracked `scripts/peak-rss.py`).
- **No source change.** Temporary instrumentation only if reverted before
  reporting and named; verify with `git status` and paste it.
- Do not modify `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` outside this
  file, or anything under `docs/dev/codex/` except appending your report.
- Leave no probe files in the repository; rum sources you generate are `*.rum`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Use `scripts/run-bounded.py --timeout SECONDS --` for long or child-spawning
  commands.
- **Stop-loss**: if you cannot settle question 1 after two serious attempts,
  stop and report what you ruled out. A precise negative is a good result.
- Paste real verbatim output for every claim.

## Report format

1. The question-1 verdict: reachable on the committed tree, or not.
2. Per-path reachability table with probes.
3. The leaked-state inventory.
4. The repair design and entrypoint contract.
5. The plan and the drafted implementation briefs.
6. Stop-loss, risks, open questions, corrections to my background.
