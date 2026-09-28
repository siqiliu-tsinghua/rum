# brief-545 — erase the unarmed cost of the option-region conversion

Implementation brief. **Do not run `git` in any form.** Everything you need
about the tree's state is in this brief.

## Goal

`7d7680de` finished the correctness of the option-region conversion: C01 and
C02 lint clean for both declared instances. It did not erase its unarmed cost.
The user ruled on 2026-09-28 that this is **erased before step 546 promotes
the region**. (Recording the first certificates, planned as 545 in brief-541,
is now 546.)

Director-measured per-iteration instructions, `taskset -c 2 perf stat -e
instructions:u`, candidate against the pre-conversion base:

| Row | Cost |
| --- | --- |
| `stringreplace-single-literal-unguarded` — every option-consuming builtin, unarmed | **+1.69%** |
| `option-explicit-delayed` | **+4.91%** vs pre-543 |
| `option-default-plus-delayed` | **+6.05%** vs pre-543 |
| `option-prefix-n` | **+6.32%** vs pre-543 |
| delayed rows, allocations | **+1 allocation, +40 bytes per iteration** |

brief-544's own report names the cause: when unarmed the owner still allocates
an `Arc<Vec<Expr>>`, clones and retains shared handles around callbacks and the
handoff, and performs membership, generation and activation-bridge work — "a
floor of this implementation, not proof that every safe alternative
representation has been exhausted". Find the alternative.

**Target:** the unarmed rows return to the pre-conversion base — for the
delayed rows the pre-543 base, for `StringReplace` the pre-544 base — in
allocations and within noise in instructions. If a floor remains, it must be
**measured and explained**, not asserted.

**Do not buy it with correctness.** The protocol's defining case stays green:
a limit that arms *later*, inside a guest callback, must still see the live
defaults and prefix. brief-544's report is explicit that simply dropping the
owner or the bridge breaks the watched defaults, adoption, handoff and
late-arm controls. Every one of them must still pass. The pattern to look for
is making the unarmed path pay nothing until an arm actually happens — deferred
materialisation at the moment of arming — rather than paying up front in case
it does.

The armed cost is **not** in scope: it is linear, about 2 300 instructions per
option plus 5 600 fixed, which the director confirmed by predicting the 8- and
32-option rows from a two-point fit within 0.12 points. It is the security
model's own path. Report it, do not tune it.

## Scope

**An objective and a fence, with a disclosure duty.**

**The objective.** Inside `crates/rum-runtime/` and `crates/rum-stdlib/`,
change what erasing the unarmed cost requires, keeping C01 and C02 lint clean
and every existing falsifier green. **Guidance:**
`runtime/accounting_mode/{options,options/driver,bridge,retained,lease}.rs`,
`runtime/options/{core,reentry,resume_frames}.rs`, and the checker's contract
records whose fingerprints the edit will touch — regenerated through the
checker's own mechanism and re-reviewed, never hand-edited to match.

**The disclosure duty.** Every touched path appears in your report with line
ranges and why.

**The fence — STOP conditions:** `rum-core`, the interner, gensym, snapshots and
the `S` model; manifests and the lockfile; `AGENTS.md`, `PROGRESS.md`,
`TODO.md`; no weakened falsifier, no `#[ignore]` on a required test, no
promotion, no change to the certified domain's semantics.

## Measurement — the gate, and it decides

1. **Pin before editing:** the five rows above plus the armed 1/8/32/128
   option-count rows, the late-arm and resume shapes, and the untouched
   neighbours brief-544 measured. Reuse its probes under
   `.temp/brief-544/probes/`. The builds, verified by the director before
   dispatch: **pre-543** `.temp/brief-543/base-fresh-target/release/rum-cli`,
   **pre-544** `.temp/brief-544/base-target/release/rum-cli`, and the current
   tree's candidate, which you build. Copy each binary to the same path in
   turn.
2. **Allocation counts first.** The delayed rows' extra allocation must be gone.
3. **Per-iteration instructions**, N and 2N, two pairs, `cpu_core` row only
   (the `cpu_atom` row is `<not counted>` on this hybrid CPU), `rum-cli` always
   given `< /dev/null`.
4. **Timing is secondary.** brief-544's timing panel disagreed with
   instructions by up to 160 points in both directions on this host. Report it;
   do not decide on it.

## Verification (MANDATORY)

Director baseline: 45 checker tests green (one ignored entry is a fingerprint
regeneration tool, not a required test), C01 0 for both instances, C02 0;
workspace 2 668 passed, 0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib options_suspension
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Nine native ZeroMQ tests cannot bind loopback in your sandbox; report them as
skipped-by-environment and do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

All scratch under `.temp/brief-545/`. Update `.temp/brief-545/PROGRESS.md` after
pinning, after each representation change with its measured rows, and at the
final gate.

## Hard rules and stop-loss

No `git`, no new `unsafe`, dependency or nightly, no traversal, no `Pending`
record as a mark. **Stop-loss keyed on progress**, as the template now states:
report the unarmed instruction deltas, the delayed allocation delta, failing
required tests and compiler errors after each attempt; any of them falling is
progress, a compile error you are fixing is not a wall, and you stop only when
two consecutive compiling attempts move none of them.

## Report format

Changed paths and ranges with why; the representation change and why it
preserves late arming; the five rows and the armed rows, pre-543 / pre-544 /
candidate, allocations and instructions; the late-arm and resume controls
green; every command with verbatim output. Append to this file and leave the
tree uncommitted.
