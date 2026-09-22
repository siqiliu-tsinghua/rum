# brief-529 — D2: the lease protocol, the activation bridge, and the option region

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** The last five briefs each ran it after being told not to;
reading this sentence is the point at which that stops. Everything you need
about the tree's state is in this brief.

## Goal

D1 built a checker that refuses unmodelled Rust and pointed it at the option
region. Its first rejection is the defect this whole line exists to close:

```
REJECTED RET001 crates/rum-runtime/src/runtime/options/core.rs:92:26-92:36
         std::vec::Vec::new
         rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol
         not a resolved function definition: std::vec::Vec::new
```

`merged_option_exprs_for_symbol` accumulates each evaluated option value into
a bare `Vec` while calling the evaluator — guest code, through a `RuleDelayed`
right-hand side — for the next one. Every value already collected is invisible
to `S` and to `W`, so an armed memory ceiling cannot see the prefix at all.
Thirty-one declarations share this path.

Close it. Build the real lease protocol, build the evaluation bridge that lets
a nested limit see an outer prefix, convert C01 and C02, and **promote the
region to Covered with zero unknowns** — the first production region this line
protects.

## What D1 established, so you do not re-derive it

- The checker lives in `crates/rum-stdlib/src/tests/retention_closure/`.
  Eight required tests pass in the ordinary gate;
  `production_obligations_are_exact` runs in 8.4 s and reconciles the whole
  workspace (672 modules, 1 023 files, 7 023 function identities).
- **The region's resolved closure is 17 bodies across 5 files with 50
  rejected edges** — against 2 633 name-spelling candidates across 349 files.
  The resolved list and every rejection are printed by that test; read them
  before planning. Most rejections are `rum-core`'s `Expr` construction:
  `Arc::new`, `OnceLock::new`, `Self::new` paths, method syntax, and a
  closure in `canonical_system_symbols`.
- `inventory/proposed-boundaries.tsv` holds six `ProposedPending` records —
  `C01-accumulator`, `C01-snapshot`, `C02-callback`, `C02-rule`,
  `C02-rule-head`, `PUBLIC-outgoing`. They are proposals with their bound
  obligations stated and **no concrete adapter**; that is your work.
- `inventory/leaves.tsv` holds five fixture-substrate records. They are the
  *shape* a trusted record must have, not production contracts.
- The accepted fixture proves the admitted surface can express an
  accumulator held across a loop under a lease, a callback result adopted
  before retention, and a transfer to an outgoing owner
  (`loops=1 adoptions=1 transfers=1`).
- The director smuggled a bare `Vec` into the covered fixture region and
  watched `RET001` fire at an exact span, then restored it green. The
  instrument works.

## Context pointers

- `AGENTS.md` first.
- `docs/dev/codex/brief-524-incremental-declaration-census.md` §4 — the `R`
  protocol design: operations, ownership, every lifetime exit, the callback
  activation bridge.
- `docs/dev/codex/brief-526-closure-mechanism-design.md` §3.3 and §3.5 — the
  admitted language and the trusted-leaf rules you must satisfy.
- `crates/rum-runtime/src/runtime/options/core.rs:87-108` (C01 and the public
  wrapper) and `:165-179` (C02).
- `crates/rum-runtime/src/runtime/accounting_mode/` — the landed mode, the
  maintained owner array, `register_accounting_builtin` at `:110`, and
  `AccountingContext::evaluate` at `:76`, which today returns a bare
  `Result<Expr, EvalError>` and is therefore **not** the bridge.
- `crates/rum-runtime/src/runtime/debug_tools/resources/maintained/` — how a
  maintained owner charges and releases, for the transfer contracts.
- `crates/rum-stdlib/src/lib.rs:74-82` — the one existing companion
  registration.

## Scope

**This section states an objective and a fence, not an enumeration.** A brief's
MAY list has been short six times on this project, and each time a correct
worker stopped on the director's omission. The enumeration below is guidance
about where the work is expected to live; it is **not** the boundary.

**The objective.** Inside `crates/rum-runtime/` and `crates/rum-stdlib/`,
change what this brief's Deliverable actually requires — including files not
named below. The admission path is a known example: the `S + W` check at
`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:154`
(`enforce_memory_limit_for_expr`) and its sibling
`enforce_active_memory_limit_for_projected_bytes` have no `R` hook, and one is
needed. Expect the suspension machinery, the maintained-owner accessors and
any kernel-API re-export to need work too.

**The disclosure duty, which replaces asking permission.** Every file you
touch that is not named in the guidance list must appear in your report's
first section with its `path:line` ranges and one line on why the objective
required it. **An undisclosed edit is the violation; a disclosed necessary
edit is not.** The director diffs the tree against your disclosure list.

**Where the work is expected to live** (guidance, not a fence):

- `crates/rum-runtime/src/runtime/accounting_mode/` — the lease and retained
  value types, the bridge, the retained construction adapters.
- `crates/rum-runtime/src/runtime/debug_tools/resources/` — the admission
  check's `R` hook and the maintained-owner accessors it needs.
- `crates/rum-runtime/src/runtime/options/core.rs` and `options.rs` — C01,
  C02 and the public wrapper.
- `crates/rum-runtime/src/runtime/suspension/` and the frame definitions, for
  a lease that is alive across a suspension.
- The call sites **the compiler names** when the public wrapper's signature
  changes — today 18 across 7 files in `crates/rum-stdlib/src/`. **Mechanical
  adaptation only**: make them compile and behave identically.
- `crates/rum-stdlib/src/tests/retention_closure/` — the promotion, the
  trusted records, the boundary statuses.
- Owning tests in `crates/rum-runtime/src/runtime/tests/` and
  `crates/rum-stdlib/src/tests/`.
- `docs/guide/resource-limits-and-sandbox.md` — only if an armed session's
  observable admission behavior changes, which it does.

**The fence — these are hard, and STOP applies to them:**

- **`crates/rum-core/`**, in particular `expr.rs`. `Expr`, `Symbol`,
  `SharedExpr` and `SharedArgs` representation stays exactly as it is. The
  retained adapter lives in the runtime, wraps the core construction it uses
  and pins that construction's semantics in its trusted record. It does not
  prove `rum-core` allocation-safe and must not claim to. If the only way
  forward looks like changing `Expr::call`, STOP and report — that is a
  different brief with a different gate.
- All `Cargo.toml` files and `Cargo.lock`. No new dependency, no toolchain
  change; the pin is stable 1.85.0.
- The interner, gensym, and the snapshot format.
- **The `S` model.** `R` never enters `S`, and `S`'s formula is not altered to
  absorb it. `S`'s *reading* may move because the accountant's own storage is
  charged — that is required, not a violation. Folding `R` into the maintained
  total to avoid a new hook is exactly the shortcut this fence exists to
  prevent.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, and `docs/` other than the one file
  named above.

## Deliverable

1. **The lease protocol**, in the runtime, with private constructors. Reserve,
   charge, release, transfer; ownership bound to the request generation; and
   every lifetime exit closed — normal publication, early return, `EvalError`,
   panic unwind, drop, suspension, resume, abandonment. A builder alive across
   a suspension transfers **exactly once**; a lease held in a suspension frame
   is a root, and `SuspensionFrame: FrameRootTrace` makes a missing traversal
   a compile error.

2. **The activation bridge.** An evaluation from inside a covered region must
   expose the live outer prefix and return a lease-bearing outcome. A nested
   `MemoryConstrained` arming *after* a prefix has accumulated must see that
   prefix. This is an acceptance condition, already ruled — a lease taken at
   entry that recorded nothing does not satisfy it.

3. **C01 and C02 converted**, expressed in the admitted surface, and the
   region **promoted**: `inventory/promoted.tsv` gains its first row,
   `production_obligations_are_exact` enforces zero unknowns inside it, and
   the six boundary records move from `ProposedPending` to a resolved status
   with their concrete adapters named.

4. **Retained construction adapters with real bounds** for the core
   construction the region uses — the `Rule` wrapper and its head. Either an
   exact literal record with an established `K` *and* a bound on simultaneous
   live instances, or a retained constructor. brief-526 §3.5 is explicit that
   there is no pre-approved numeric `K`; if you cannot establish one
   mechanically, use the retained constructor. Do not invent a constant to
   get the gate green.

5. **No raw egress.** The public wrapper does not return a bare `Vec<Expr>`
   any more. A covered region reaching a raw boundary must be a checker
   rejection, not a convention. Rum has no downstream consumers and the API
   is deliberately unlocked, so break it.

6. **The four escaping consumer copies are D3's, and must be visible.**
   `effective_same_test` clones an `Expr` at
   `crates/rum-stdlib/src/predicates/collections.rs:360` and
   `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:535`; `AlignColumns`
   copies a callback string at `.../strings/scripting/text_shape.rs:229`;
   `UrlParse` copies an option-derived scheme at
   `.../strings/scripting/url.rs:202`. **Do not fix them here.** Record them
   as unresolved consumer obligations in the inventory so the promotion
   cannot be read as covering them.

## Falsifiers — watched, not asserted

- **`compile_fail` proofs** on the protocol API, in the owning rustdoc, each
  with a compile-*pass* control built first: a forged or private lease
  constructor, a reused moved retained value, an unleased raw extraction, an
  invalid frozen transfer, and a branded handle escaping its lifetime. Pin the
  expected error where the compiler gives a stable code; a `compile_fail` that
  passes for the wrong reason is a false positive waiting to happen.
- **The activation falsifier**, which is the protocol's whole point:
  accumulate a large prefix through delayed option evaluation **while
  unarmed**, arm a nested limit inside a later callback, and show admission
  trips — with a control that has no retained prefix and does not trip. A
  final-result-only rejection does not discharge this.
- **The region falsifier**: remove one lease inside the promoted region and
  watch `RET001` fire with its span, then restore. Same for an unadopted
  callback result and `RET004`.
- **The live rum-native falsifier**: a `RuleDelayed` option whose right-hand
  side is large, evaluated under an armed limit, refused before unbounded
  progress rather than after the final result exists.

Each one: watched red with its pinned diagnostic, then green. Save complete
output to a file under `target/brief-529/` before quoting it.

## Measurement — counters first, timing second

This host's timing lane is unreliable: its P and E cores are 1.27x apart, and
a single pair cannot gate the panel.

- **Primary gate: allocation counts**, via
  `scripts/core-memory-microbench.sh` with a rum-cli that emits the
  alloc-count profile. Allocation counts are load-immune. **Report counters
  first, and if they do not move, do not quote a time.**
- **Secondary: `scripts/core-language-microbench.sh`**, base and candidate
  built at the same path, four passes each, profiling env OFF. Normalize
  against **named untouched labels**, never the suite median. Report the
  option-consuming labels — string replacement, string cases, `Union`,
  `DeleteDuplicates`, sorting with a comparator — **and** their same-tier
  neighbors, and any other hot-label movement you see.
- **The unarmed path is where the hard line lives.** The line's cumulative
  unarmed cost is already `+0.965% / +3.239% / +1.018%` from A-E. Report your
  step's delta against both your own base and that pre-line base. A bar is a
  data point, not a stop condition: if it costs more than expected, report the
  curve — do not stop, and do not tune it away silently.
- If the unaffected control labels move as much as the affected ones, that is
  the noise floor. Say so and do not "fix" noise.

## rum / Rust pitfalls (read twice)

1. **Evaluator flows are suspension-safe.** Preserve the suspend/resume
   continuation shape; do not collapse a `suspend_with_frame*` into an eager
   call to simplify the ownership story.
2. **A lease in a frame is a root.** A missing `FrameRootTrace` traversal is a
   compile error by design — do not work around it by keeping the lease out
   of the frame.
3. **Shared children are multiple occurrences.** Transferring one `Arc` clone
   does not discharge the others.
4. **A callback's result size is unknown before it returns.** No honest
   precheck covers it; adopt it before retention and before the next step.
5. **Hot paths avoid owned-tree clones.** Prefer borrowed traversal and `Arc`
   clone over deep clone on anything that runs per evaluation step.
6. **The checker's rejection is information, not an obstacle.** If the
   admitted surface cannot express something the region genuinely needs, that
   is a finding for the report — not a reason to widen the trusted set or to
   add a module allowlist.
7. **`rum-core` is read-only here.** If the only way forward seems to be
   changing `Expr::call`, STOP and report; that is a different brief with a
   different gate.

## Verification (MANDATORY — you run it; iterate to green)

Timeouts derive from director measurements on this host, warm tree, taken
immediately before dispatch: `cargo test -p rum-stdlib` green in **21 s**,
`cargo test --workspace` green in **35 s**, `production_obligations_are_exact`
in **8.4 s**. The tree is green at your starting point; any red you see is
yours.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Then the measurement runs. Paste verbatim: all eight checker tests, the test
totals, every falsifier transcript, the allocation-count table, and the
per-label rows for the option-consuming labels and their neighbors.

Nine native ZeroMQ tests cannot bind loopback in your sandbox. That is known
and is the director's to settle — report them as skipped-by-environment and
**do not** set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` or otherwise mask them.

## Hard rules

- **No `git`, in any form.** Stated at the top; repeated here because it has
  been ignored five times.
- Do NOT commit, branch, push, or touch git config. The director gates and
  commits.
- Respect the fence in Scope — `crates/rum-core/`, every `Cargo.toml`, the
  interner and snapshot, and the `S` model. Those are STOP conditions. Outside
  the fence, the rule is the **disclosure duty**, not permission: change what
  the objective requires and list every unnamed file you touched.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No default-path cost for off-by-default machinery.** The lease operations
  erase when unarmed, on the mode seam A-E built. Name where that erasure is
  proven.
- No permissive unknown mode, no `#[ignore]`, no feature gate on a required
  test, no cap that silently drops an edge, no trusted entry added to make a
  gate green.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- Scratch goes under `target/brief-529/`.
- **Stop-loss, keyed on progress.** Report two numbers after each attempt:
  unknowns remaining inside the promoted region, and required gates passing.
  Stop only when **two consecutive** attempts move neither. A compile error
  you are actively reducing is not a wall, and a checker rejection is the
  instrument working.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. The protocol: operations, ownership, and how each lifetime exit is closed,
   with the `path:line` of each.
3. Verification — every command with verbatim output; the falsifier
   transcripts, each red-then-green.
4. Measurement — allocation counts first, then timing with its controls, and
   the unarmed delta against both bases.
5. The promotion: the region, its zero-unknown evidence, the resolved
   boundary records, and the consumer obligations left explicitly open.
6. Stop-loss / open questions.
