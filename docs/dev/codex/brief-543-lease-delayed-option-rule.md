# brief-543 — lease the delayed-option rule and its continuation: C02 to 0

Implementation brief. **Do not run `git` in any form.** Everything you need
about the tree's state is in this brief. (brief-542 was the first worker on
this line to run no git command at all; keep that.)

## Goal

Second of brief-541's four steps (plan in
`docs/dev/codex/brief-541-option-region-conversion-plan.md`, report from line
165, draft 543 at its §7). `70a9edf9` left C01 at 17 and C02 at 15. Convert
**C02** — `Session::evaluate_runtime_option_expr`
(`crates/rum-runtime/src/runtime/options/core.rs:248-265`) — and the rule-resume
boundary (`RuntimeOptionSuspensionFrame`) onto the lease protocol.

**Predicted end state, written before any work: C01 17, C02 0**, including any
callback-result and B12 preconditions the conversion exposes. Production
promotion stays 0; recording certificates is step 545. C02's outward signature
is preserved, so this step does not silently rewrite the accumulator boundary
that step 544 owns.

## What to build

- **A private two-child rule builder** with a clone-and-adopt / result
  handoff, and contract records **B10** (`Expr::call`) and **B11**
  (`system_expr("Rule")`) with exact caller preconditions and falsifiers. B10's
  runtime-enforced guarantee belongs to the guarded adapter's domain only — do
  **not** globally discharge bare `Expr::call`. Pin the literal `Rule` contract.
  A cloned `Expr` stays `OwnedUnleased` until actually adopted; the clone's
  known type is not a capacity receipt (director ruling U4).
- **Evaluate the right-hand side through the activation bridge**, adopt the
  result before any construction or growth, and preserve error payloads —
  `Err` can carry `Expr`s.
- **Convert `RuntimeOptionSuspensionFrame`** to the private lease-bearing
  transfer model: lazy name capture preserved, a charged result on resume even
  with another `Arc` owner alive, transfer exactly once.
- **`resume_or_suspend!`**, defined at `runtime/suspension/core.rs:784-800`: a
  narrowly scoped source adapter whose definition, arm and hygiene must all
  match — not a blanket macro admission. If you replace the macro in C02 with a
  typed match, keep the macro-equivalent branch order and still build the
  adapter and its controls. Raw frame insertion is not kept for its spelling;
  B12's domain excludes raw frames even when the callee name matches.

Do not claim C02 fixed by moving its raw lifetime into an unproved helper. Any
helper that receives hazardous ownership gets a discharged concrete boundary.
`finish_list` is not a rule builder — do not build a temporary `List` and peel
it to evade the protocol. Do not eagerly build a frame or a name clone on the
success or ordinary-error paths.

## The acceptance assertion changes — director-authorized

Update `local_production_acceptance` so it **requires C02 clean**, keeping
C01's rejection, the rejected raw-container negative fixtures and
`LeaseProbe538` clean. This is a deliberate tightening the director approves;
no other change to that test is permitted, and no assertion may be loosened.

## Required runtime outcomes

Non-`Call` and immediate `Rule`; delayed success; repeated and nested input;
ordinary error; guest panic; a shared frame; abandonment and reset; memory
refusal; overflow; and **late arming** — a limit that arms after the prefix
exists must see it. Each with a falsifier watched red, then green.

## Measurement — this is a hot path

Production changes on the option path, which every option-consuming builtin
takes. The discipline this line earned:

1. **Allocation counts first**, on the four exact `StringReplace` rows and
   added delayed / rule / suspend fixtures. The existing memory script has no
   option consumer (director ruling U5 authorizes adding these).
2. **Per-iteration instruction slopes** for base and candidate:
   `taskset -c 2 perf stat -e instructions:u` at N and 2N, copying both
   binaries to the same path in turn. Take the `cpu_core` row; the `cpu_atom`
   row is `<not counted>` on this hybrid CPU. Pass `< /dev/null` to `rum-cli`.
3. **Timing** via the language microbench only as a secondary panel. Timing on
   this host has reported 7-12% regressions that instruction counts refuted;
   if timing disagrees with instructions, report both, do not rename either.

Pin probes before edits, store every `.rum` file and raw output under
`.temp/brief-543/`, state run caps and timeouts derived from a measured run,
and never measure alongside CPU-heavy tests. A real regression is fixed or
reported — and under the standing ruling a bar is a data point, not a stop.

## Scope

**An objective and a fence, with a disclosure duty.**

**The objective.** Add the private charged rule-construction and result
boundary, convert C02's synchronous and resumed paths, and implement only the
local admissions, concrete contracts, tests, documentation and measurement
support needed to prove it, including registration and mark-invalidation
plumbing. **Guidance:** `options/core.rs:248-265` and the frame own rule
evaluation and resumption; `accounting_mode/{retained,bridge,frozen}.rs` own
receipts, adoption and freezing; the checker's `local/` modules and
inventories own local evidence.

**The disclosure duty.** Every touched path appears in your report with line
ranges and purpose — especially new runtime adapters and checker modules.

**The fence — STOP conditions:** no C01 / B01 / B02 accumulator or consumer
rewrite (step 544); no arbitrary macro or trait admission; no promotion;
`rum-core`, the interner, gensym, snapshots and `S` untouched; no manifests or
lockfile; no `AGENTS.md`, `PROGRESS.md`, `TODO.md`. Public option behaviour is
preserved.

## Verification (MANDATORY)

Director baseline: 41 checker tests green, C01 17, C02 15; workspace 2 639
passed, 0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib options_suspension
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Before edits, save the 17/15 → 17/0 prediction, all 15 remaining C02 row IDs,
and the base binary and source identity under `.temp/brief-543/`. Compare fresh
counts, not the old macro scanner's partial rows. Nine native ZeroMQ tests
cannot bind loopback in your sandbox; report them honestly as
skipped-by-environment and do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

`.temp/brief-543/PROGRESS.md` after the rule builder, the synchronous
conversion, the frame conversion, and the final gate: owner and exit matrix,
current counts, red/green controls, raw measurement paths, tree state and the
exact next action.

## Hard rules

No `git`, no new `unsafe`, dependency or nightly, no external-engine oracle,
no blanket macro admission, no `Pending` record as a mark, no weakened
assertion, no traversal. No change to `Expr` / `Symbol` representation or `S`.
Testing Policy is a hard line; rustc as an oracle for Rust facts about this
repository is a director ruling, not its banned shape. Stop-loss keyed on
progress: two attempts with neither a better falsifier nor a lower count
trigger a report, not a broader admission.

## Report format

Changed paths and ranges with why; commands with verbatim checker and test
output; 17/0 predicted versus measured as located rows; B04/B10/B11/B12 domains
and exit ownership; allocation, instruction and timing tables for targets and
peers; residual questions and stop-loss. Append to this file and leave the
tree uncommitted.
