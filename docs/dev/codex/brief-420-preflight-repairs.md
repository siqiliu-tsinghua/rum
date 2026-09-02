# brief-420 — make the eighteen honest, or say why they cannot be

Shape: **implementation**. Worker: **cdx (Codex)**.

Gap B, step 3. brief-419 measured; this repairs. Its allowlist
(`eval_dispatch/mod.rs:7`) is the work list, and it is the gate: delete an
entry, and the falsifier immediately checks whether your repair is real.

## Goal

Every entry in `KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS` either

- **gains a real byte projection before it allocates**, and leaves the list, or
- **stays, with a recorded reason** stating why no sound and useful up-front
  projection exists for it.

An empty list is not the target. A list whose every remaining entry carries an
argued reason is.

## The one rule that matters most

**No reclassification. None.** Do not move a builtin from `Preflight` to
`Incremental`, `Delegated`, or anything else in this brief, even when you are
confident that is where it belongs.

This is not caution, it is a specific defect I am refusing to repeat.
brief-417 reclassified eleven builtins out of `NonAmplifying`; five of them —
`Append`, `Commonest`, `DeleteDuplicates`, `DeleteMissing`, `Union` — landed in
`Preflight`, which at that moment had no enforcement, and brief-419's falsifier
caught all five the day it gained teeth. Moving a builtin out of an enforced
contract into an unenforced one is a deferral wearing a repair's clothes, and
neither of us noticed at that gate. `Incremental` has no falsifier yet. Until it
does, a move into it is invisible, and I will not accept one.

If your honest conclusion for a builtin is "this is `Incremental`, not
`Preflight`", **write that as the recorded reason and leave it in the list**.
That is the right answer and I want it — as a sentence, not as an edit.

## Context pointers

- `AGENTS.md` first.
- `docs/dev/codex/brief-419-preflight-falsifier.md`, appended report §4 — the
  table, with each builtin's observed result and argument bytes and the test
  that surfaced it. Your starting evidence.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-190`
  — `enforce_active_memory_limit_for_projected_bytes`, the choke point every
  repair must go through. It no-ops when no ceiling is armed, and it records the
  projection for the falsifier before that check.
- `crates/rum-stdlib/src/strings/text/builtins.rs:660-668` (`StringRepeat`) and
  `:753-757` (`CharacterRange`) — the two payload-aware projections already in
  the tree. Copy their shape.
- `docs/dev/codex/brief-414-memory-accounting-gaps.md`, appended report §3 —
  the inventory, for growth factors.

## Two things I already determined, so you do not spend the run on them

**`Range` is a false positive and needs a different fix.**
`enforce_range_memory_constraint` (`structure_ops/sequence/range.rs:28-39`)
returns at `:29-31` when no ceiling is armed, so it never reaches its
projection and the falsifier cannot see it. Its projection is real and does run
in production. I grepped the tree: it is the **only** site with that shape.

Fix it so the falsifier stops reporting it, **without adding cost to the
unarmed path** — note that `bigint_count_to_usize` does a `to_string().parse()`,
which is not free, and `Range` is used in loops. Recording the projection
before the early return is one way; find the cheapest one and say what you
chose. Then remove `Range` from the allowlist.

**My expectation about the rest, which I want you to confirm or refute.** Seven
look genuinely projectable from their arguments: `Append` (input plus one
element), `Union`, `DeleteDuplicates`, `DeleteMissing`, `Commonest` (all
bounded by the sum of their inputs), `Power` (computable bit length), and
`Diff` (bounded by both sequences). Ten look like decoders and tokenizers whose
output size depends on content, not input length — the six `From*` codecs,
`RumTokens`, `TokenizeWith`, `UrlParse`, `ParseArgs` — where any sound bound is
`input_bytes` times a per-node constant, which is loose enough to refuse work
that would have succeeded.

**That is my reading from the outside and it may be wrong.** Check each one and
say so if it is. I would rather be corrected than agreed with.

## The hazard that decides how loose a projection may be

A projection that over-estimates refuses work that would have fit. That is a
usability defect this line has no licence to introduce, and it is harder to
notice than an under-count because nothing fails loudly — a script just gets
`TerminatedEvaluation["MemoryLimit"]` where it used to get an answer.

So for **every** builtin you repair, add both halves:

1. a low-ceiling test proving refusal happens **before** the large allocation —
   check the process high-water mark, not just the returned error, the way
   brief-413's `StringRepeat` probe did; and
2. a **realistic-ceiling test proving an ordinary input still succeeds**, sized
   so that a projection which is loose by more than a small factor would fail
   it.

The second is the one that matters here. If you cannot write it for a builtin,
that builtin's projection is too loose and it belongs in the "recorded reason"
column instead.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` — the allowlist and its
  recorded reasons only
- `crates/rum-runtime/src/runtime/arithmetic/canonical.rs` — `Power`
- `crates/rum-stdlib/src/structure_ops/sequence/range.rs` — `Range`
- `crates/rum-stdlib/src/structure_ops/surface.rs`, `ordering_sets.rs`,
  `dedup.rs` — `Append`, `Union`, `DeleteDuplicates`
- `crates/rum-stdlib/src/associations/missing.rs` — `DeleteMissing`
- `crates/rum-stdlib/src/arithmetic/stats.rs` — `Commonest`
- `crates/rum-stdlib/src/strings/similarity/mod.rs` — `Diff`
- `crates/rum-stdlib/src/{csv,json,ini,toml,xml,yaml}.rs` and their existing
  decode submodules
- `crates/rum-stdlib/src/strings/tokenize.rs`,
  `strings/scripting/url.rs`, `strings/scripting/args.rs`
- the module roots of anything above whose exported types change
- test files and test module roots in either crate
- this brief, to append your report

A file that needs **only** a projection call added is expected and in scope
even if I did not name it. Anything more than that, **STOP and report**.

Files you MUST NOT touch: the growth classifications
(`catalog/memory_growth_contract.rs`, `tests/memory_growth_contract.rs`'s
inventory rows), `debug_tools/resources/memory.rs`, any `ResourceLimits`
default or the stride, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except
appending your report, `.git`, git config.

## Hard rules

- **No reclassification**, per the section above. This is the stop-loss I care
  about most.
- **Do not loosen the falsifier**, its allowance, or the meaning of the
  allowlist to make an entry disappear. An entry leaves the list only by the
  handler actually projecting bytes.
- **No default-path cost.** A projection call on an unarmed session must cost
  what the existing eighteen cost: the choke point's own early return. If a
  repair needs an expensive size computation, that computation must not run when
  no ceiling is armed — `Range` is the worked example.
- **Saturating or checked arithmetic only.** A projection that overflows must
  fail closed, never wrap to a small number.
- **Do not weaken an existing generated-element or integer-work bound.** Add
  the byte projection alongside; those bounds stay.
- **No new `unsafe`.** Do not weaken an existing test.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-419 gate. If your sandbox denies it and
  you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so and mark those nine
  unobserved.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline -p rum-stdlib
cargo test --offline -p rum-runtime
cargo test --offline --workspace
```

The workspace run must be green with your shortened allowlist. Since the
falsifier is armed structurally in test builds (brief-418), a repair that does
not actually project will fail it — that is the gate, and it is automatic.

Then a microbench A/B, base against candidate in one window, because you are
adding calls to builtin paths:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the rows for the labels touching the repaired builtins and their
same-tier neighbours. **Name your control labels before looking at the
candidate numbers.** A whole-run offset moves every label together and a
lopsided direction split is not evidence. Profiling env off for every timing
number.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Per builtin, a table row: repaired or retained, the projection formula if
   repaired, the recorded reason if retained.
3. Whether my seven-versus-ten reading was right, with what corrected it.
4. What you did for `Range`, and how you know it costs the unarmed path
   nothing.
5. The realistic-ceiling tests: what input each admits, and how you sized them
   to catch an over-loose projection.
6. The final allowlist contents.
7. Verification — every command, verbatim output, plus the A/B rows with named
   controls.
8. Stop-loss / open questions.
