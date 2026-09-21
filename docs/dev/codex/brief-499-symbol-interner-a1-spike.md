# brief-499 — spike: what A1 actually costs (non-`Copy` `Symbol`, owned text)

## Goal

The 2026-09-02 user ruling settled that the process-global leaking symbol
interner becomes **session-scoped (option A1)** — but ruled that A1 opens as a
**spike, not an implementation**, because A1 makes `Symbol` non-`Copy` and
`Symbol` is the key type of `definitions`, `attributes` and six caches. This
brief is that spike. Build a throwaway prototype of A1's *representation*,
and return the three numbers the ruling made A1 conditional on:

1. the **true** compiler fallout of removing `Copy`, per crate, classified;
2. the candidate `size_of::<Symbol>()` / `size_of::<Expr>()`;
3. the **instruction-count** cost of the new equality, hash, ordering, clone
   and drop paths on symbol-hot workloads, against an ordinary one.

**Nothing here lands.** Leave the tree uncommitted; the director decides
whether A1 proceeds on these numbers or the line falls back to A2. Do not
write a recommendation as if it were a decision — report the numbers and say
what they do and do not establish.

## Why a spike, and the one variable it holds

`Symbol`'s `PartialEq`, `Hash` and `Ord` are all built on `id` being a perfect
collision-free hash of the text: `eq` is a fast **accept** on `id`
(`crates/rum-core/src/expr.rs:250-254`), `Ord` uses `id` equality as a fast
path (`:271-285`), `Hash` hashes `id` alone (`:258-263`). A content
fingerprint collides, so equality must demote to
fingerprint-rejects-then-compares-text. That demotion, plus `Arc` clone/drop
replacing a 24-byte `memcpy`, is what A1 costs. Everything else A1 buys
(reclaim on `reset()`, per-session attribution) is plumbing on top.

**So this spike changes the representation and NOT the ownership.** Keep the
interner process-global and still leaking for the duration of the spike. If
the representation alone is too expensive, A1 falls on that number and the
session plumbing never has to be built; and mixing both changes in one
measurement makes the number unattributable. **Do not build a session-owned
interner, a reclamation path, a canonical-pool redesign, or any memory-ledger
accounting in this brief.**

## Context pointers

- `AGENTS.md` — operational notes, the Testing Policy hard lines, the
  hot-path-changes-are-measured rule.
- `docs/dev/codex/brief-414-memory-accounting-gaps.md` §"Gap A", and the
  Report's §2 (options table, the A1 recommendation, its seven rules). That is
  the design pass this spike is testing; **its microbench line-number
  citations are stale** — re-derive any label location yourself.
- Key files:
  - `crates/rum-core/src/expr.rs` — `SymbolText` (`:22-27`), `Symbol`
    (`:29-39`, `Copy` derived at `:35`), `SymbolInterner` (`:41-44`),
    `symbol_interner` (`:47-55`), `intern_text` (`:57-70`, `Box::leak` at
    `:66`), `intern_symbol_text` (`:80-96`, the `u32` `expect` at `:88`),
    `canonical_system_symbols` (`:98-105`), the constructors (`:108-138`),
    `interned_id` (`:164-168`), `PartialEq`/`Hash`/`Ord` (`:250-285`), serde
    (`:287-320`), `Atom::Symbol` (`:536-542`), `Expr` (`:749-753`),
    `Expr::symbol`/`contextual_symbol`/`system_symbol` (`:801-812`).
  - `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:70` — the
    only production consumer of `interned_id`, feeding a Fibonacci-hashed
    bloom.
  - `scripts/core-language-microbench.sh` — the source of the exact probe
    statements named below.

## Scope

**Files you MAY modify** — this is a disclosure duty, not a list:

- `crates/rum-core/src/expr.rs` and `crates/rum-core/src/heads.rs`.
- **Any `.rs` file under `crates/` that the compiler forces you to change** to
  make the workspace build again after `Symbol` stops being `Copy`. Every such
  file must appear in your report with the error kind that forced it (see the
  D1 classification). **A change that is not compiler-forced is a stop** —
  write it in the report instead of making it.
- Probe workloads go under `target/brief-499/` as `*.rum` files (the repo
  pattern; `.rum` suffix is a hard rule in `AGENTS.md`).

**MUST NOT touch**: `scripts/**` (the microbench panel stays byte-identical so
both arms run the same suite), `docs/**` except this file's own report section,
`AGENTS.md`, `TODO.md`, `PROGRESS.md`, `Cargo.toml`, `Cargo.lock`, `.git`.

**MUST NOT do**: build session ownership or reclamation; delete the leak; add
interner bytes to the memory walk; change serde wire shape; change any
documented language behavior. Those are the follow-up's, and doing them here
destroys the measurement.

## Stages — write each stage's evidence into your report BEFORE starting the next

A run can be cut off mid-write. Stage 1's number is the single most
load-bearing deliverable and it is independent of everything after it, so it
must be on paper before you attempt the prototype.

### Stage 1 — the fallout count, with nothing repaired

Delete `Copy` from the `Symbol` derive at `crates/rum-core/src/expr.rs:35`.
Change nothing else. Then count.

**The instrument lies if you read it naively, and this is the trap that would
invalidate the whole stage.** `cargo check --workspace` compiles in dependency
order and **stops at the first crate that fails**, so a single workspace error
count is only `rum-core`'s. The quantity asked for is per crate, each measured
once its dependencies compile. Dependency order here is: `rum-core` →
`rum-syntax` → `rum-runtime` → `rum-stdlib` → `rum-host` → `rum-terminal` →
`rum-repl` → `rum-cli` → `rum-jupyter`.

So: repair `rum-core` to compiling, record its count, then move to the next
crate, and so on. Report:

| crate | errors, lib only | errors, `--all-targets` | dominant error kinds |
|---|---|---|---|

Use `cargo check -p <crate>` and `cargo check -p <crate> --all-targets`; tests
are part of the repair cost, so both columns are required. Classify every
error by kind with one representative `path:line` each — at minimum: use of
moved value, `.copied()` on an iterator of `&Symbol`, dereference move out of
a map/slice, pattern binding by value, struct field requiring `Copy`, a trait
bound demanding `Copy`, array/`[T; N]` initialization. Add kinds you find.

State plainly what your total counts and what it does not: brief-414's Report
claims "260 direct constructor expressions"; a regex over the four public
constructors (`Symbol::new|system|from_name|with_identity`) returns **179**
call sites today. Neither is the non-`Copy` fallout. Say which quantity your
table is.

### Stage 2 — the prototype representation

Replace the representation, keeping the global interner:

```rust
pub struct SymbolText { fingerprint: u64, context: String, name: String }
pub struct Symbol { fingerprint: u64, identity: Option<NonZeroU64>, text: Arc<SymbolText> }
```

- The interner becomes `HashMap<(String, String), Arc<SymbolText>>` (or
  equivalent) holding **strong** `Arc`s. It still never reclaims. That is
  deliberate for this spike.
- **The fingerprint must be a fixed, seedless, process-stable hash** over
  `(context, name)` — FNV-1a or equivalent written inline, no `RandomState`,
  no `DefaultHasher`. The ruling redefines `interned_id` as a *stable content
  fingerprint*, whose promise is stronger than today's: same text ⇒ same value
  across processes, not merely within one.
- `eq`: `identity` equal AND (`Arc::ptr_eq` fast accept OR fingerprint equal
  AND context and name equal). Fingerprint equality alone is never an accept.
- `hash`: fingerprint and identity, mixed as today (`:258-263`) so the
  distribution the bloom depends on is unchanged.
- **`Ord` is the correctness trap.** Today's fast path returns on `id`
  equality; a fingerprint *collision* would then order two different symbols
  as `Equal` and silently corrupt every sorted structure. The fast path must
  become `Arc::ptr_eq`, and the fallback stays exactly today's
  `context.cmp` → `name.cmp` → `identity.cmp`. Ordering semantics must not
  move by one pair.
- `interned_id()` keeps its signature and returns the low 32 bits of the
  fingerprint; rewrite its rustdoc so it no longer claims a dense id. The
  bloom at `stability.rs:70` already multiplies by the golden-ratio constant,
  which is Fibonacci hashing and is unharmed by receiving a hash.
- Delete the `u32::try_from(pairs.len()).expect(...)` panic path at `:88`; it
  has no successor under a fingerprint.
- Serde stays text-based and byte-identical (`:287-320`).
- `canonical_system_symbols` stays a process-static map, now of `Arc`-backed
  symbols, so `Symbol::system` costs one `Arc` clone per lookup. That cost is
  part of what you are measuring — do not special-case it away.
- **No `unsafe`, for any reason, including to preserve a layout.**

Then repair the workspace to green.

### Stage 3 — the sizes

Report, for base and candidate: `size_of::<Symbol>()`, `size_of::<Atom>()`,
`size_of::<Expr>()`, `size_of::<SharedExpr>()`. Today's observed values are
24 for `Symbol` and 32 for `Expr`; confirm rather than assume.

**If `Expr` grows, stop and report before measuring anything else.**
`size_of::<Expr>()` is read by the memory model at ten-plus stdlib accounting
sites, so a growth moves the ceiling's own arithmetic and is a director
decision, not a cost to absorb.

### Stage 4 — the cost

Primary instrument is instruction counts, not the timing panel. The panel
cannot resolve a question this size — on this host its own untouched controls
moved +3.70% to +12.45% in a recent run, while this recipe repeats to 0.003%:

```
taskset -c 2 perf stat -e instructions:u -x, -- <rum-cli> <workload>.rum < /dev/null
```

Read `cpu_core/instructions/u`, require `100.00` multiplex coverage, release
profile, **profiling env OFF**, both arms built at the same path, three runs
per arm, judge every delta against the observed within-arm spread. Run with
stdin closed or the probes hang.

Four workloads, each a `.rum` file under `target/brief-499/`, each stated
verbatim in the report:

1. **Cold intern** — dominated by first-time interning of distinct names.
   `Symbol[StringJoin["...", ToString[i]]]` over a `Table` is the shape the
   panel already uses (`scripts/core-language-microbench.sh:884-891`).
2. **Warm intern** — the same names re-interned, so the map hits and the cost
   is lookup plus `Arc` clone rather than allocation.
3. **Symbol-hot evaluation** — lift the EXACT statements of these labels from
   the script and run them at their own iteration counts, nothing added around
   them: `literal-integer` (`:479`, the control), `symbol-ownvalue-read`
   (`:481`), `set-ownvalue` (`:487`), `module-local` (`:497`),
   `downvalue-exact-hit` (`:530`), `matcher-single-tiny` (`:569`),
   `replaceall-simple` (`:586`), `sort-symbol-default-width4000` (`:982`) and
   `...width16000` (`:988`), and the two
   `valueq-undefined-symbol-definitions-*` labels. This is the set where a
   non-`Copy` `Symbol` should hurt most: map keys, clone/drop, ordering.
4. **Evaluation-dominated ordinary** — a small program doing real arithmetic
   and list work, where symbol construction is a vanishing fraction. This is
   what almost every real program looks like.

Separately, with the allocation counters armed (their own runs, never mixed
with a timing or instruction run), report allocation counts per workload for
both arms. A count that moves on workload 3 or 4 is a finding in itself.

**Report the split; do not tune.** Acceptance is the director's call and it
will be judged the way this project's precedents judge it — a cost confined to
the shape the change is about, with the ordinary case unmoved, is a different
verdict from a cost on workload 4. If a number looks wrong, say so; do not
silently re-measure everything.

## Deliverable

- The Stage 1 per-crate table, classified, with the quantity named.
- The prototype in the working tree, workspace green.
- The Stage 3 size table, base and candidate.
- The Stage 4 instruction-count table: four workloads × two arms × three runs,
  with within-arm spread, plus the allocation-count table.
- A closing section stating explicitly what the spike does **not** establish:
  session ownership, reclamation on `reset()`, escaped-value lifetime,
  cross-session equality cost (within one session `Arc::ptr_eq` always hits;
  two sessions holding the same text would not, and that path is the
  follow-up's ingress-import problem).

## Verification (MANDATORY — you run it, and paste it verbatim)

```
cargo fmt --check
cargo test --workspace
```

Green is required before any Stage 4 number is credited: a cost measured on a
build whose semantics moved is not a cost. If a test fails and you believe the
test encodes the old `id` semantics rather than a real contract, **stop and
report it** — do not weaken it. Two tests in the tree could not be made to
fail under any preintern count (recorded in `TODO.md` §"Opened by the
2026-08-31 consistency review"); if one of those is what you hit, say so.

## rum / Rust pitfalls (read twice)

1. A probe is the label's EXACT statement — no sink `Set`, no `Print`, no
   wrapper. If a loop might be elided, raise the iteration count.
2. One variable per measurement. The representation changes; the ownership
   does not; the microbench script does not.
3. Hot paths avoid owned-tree clones. `Arc` clone over deep clone, borrowed
   traversal where the compiler offers it — but **do not "fix" a non-`Copy`
   error by cloning the whole enclosing `Expr`**; that is a silent hot-path
   regression wearing a compile fix's clothes. Prefer a borrow; if you must
   clone a `Symbol`, that is the cost being measured and it is fine.
4. Evaluator flows are suspension-safe: if the repair reaches an `eval_*` /
   `resume_*` path, preserve the suspend/resume continuation shape.
5. Derive timeouts from data, and use
   `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything long
   or child-spawning.
6. Stop-loss is real: if the same crate fails to repair twice for the same
   reason, STOP and report with the count you have.

## Hard rules

- Do NOT `git commit`, branch, push, stash, `git checkout`, or touch git
  config or the index. Leave everything in the working tree. (`git checkout`
  restores from the index, which on your own uncommitted output is a delete.)
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  it in the report.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`. Not for
  layout, not for a lifetime, not anywhere. If you are convinced it is
  unavoidable, STOP and report.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no third-party engine comparisons.
- No network. Touch nothing outside this repo. The only commands are file
  edits within Scope, `cargo`, `perf`/`taskset`, the release `rum-cli`, the
  `scripts/run-bounded.py` wrapper, and `python3` for parsing.
- Paste real verbatim output. If a stage did not reach green or did not
  produce its evidence, say so explicitly — never fake or interpolate a
  number.

## Report format (your final message, and appended to this file)

1. **Stage 1** — the per-crate fallout table and the error classification,
   written before anything else was attempted.
2. **What changed** — per file, line ranges, one-line why, each tagged with
   the error kind that forced it (or "representation" for the four core
   files).
3. **Verification** — each command with verbatim output (test counts, fmt).
4. **Stage 3 sizes** and **Stage 4 numbers** — the tables, with spreads.
5. **Verdict** — what the numbers say about A1 versus the A2 fallback, stated
   as evidence rather than as a decision, plus the explicit list of what the
   spike does not establish.
6. **Stop-loss / open questions** — where you stopped and why; empty if none.

---

## Execution report (2026-09-21; staged evidence)

### 1. Stage 1 — non-`Copy` compiler fallout (STOPPED; incomplete)

The first source edit removed only `Copy` from `Symbol`'s derive. No
representation change has been attempted. Checks run offline under
`scripts/run-bounded.py`; diagnostics are collected with
`--message-format=json`. Counts describe compiler errors from removing
`Copy`, not constructor expressions. Brief-414's 260 constructor expressions
and this brief's 179 four-constructor call sites are different quantities.

| crate | errors, lib only | errors, `--all-targets` | dominant error kinds |
|---|---:|---:|---|
| rum-core | 1 | at least 1; tests blocked by runtime dev-dependency | E0507: dereference move from shared map value |
| rum-syntax | 0 | not reached (runtime dev-dependency) | none in library |
| rum-runtime | 222 | 526 emitted; 304 distinct across lib/lib-test | E0204, E0277, E0599, E0382, E0507, E0508 |
| rum-stdlib | not reached | not reached | not measured |
| rum-host | not reached | not reached | not measured |
| rum-terminal | not reached | not reached | not measured |
| rum-repl | not reached | not reached | not measured |
| rum-cli | not reached | not reached | not measured |
| rum-jupyter | not reached | not reached | not measured |
| rum-derive (additional workspace member) | no separate check | no separate check | not measured separately |

Initial `cargo check -p rum-core` and `cargo check -p rum-core --all-targets`
both emitted one E0507 at `crates/rum-core/src/expr.rs:134`, `return *symbol`.
The compiler's verbatim diagnostic was:

```text
error[E0507]: cannot move out of `*symbol` which is behind a shared reference
   --> crates/rum-core/src/expr.rs:134:20
    |
134 |             return *symbol;
    |                    ^^^^^^^ move occurs because `*symbol` has type `Symbol`, which does not implement the `Copy` trait
    |
help: consider cloning the value if the performance cost is acceptable
    |
134 -             return *symbol;
134 +             return symbol.clone();
    |
```

Dependency qualification: both rum-core and rum-syntax declare rum-runtime
as a dev-dependency. Their complete `--all-targets` counts therefore require
rum-runtime to compile; the initial core all-targets failure is a lower
bound, not a complete tests count. Test targets were not revisited because that dependency did not reach green. The listed dependency order also omits rum-derive,
which is a workspace member and runtime dependency.

Runtime's unrepaired check (after only the core repair) emitted 222 library
errors: E0204=28, E0277=25, E0599=10, E0507=82, E0382=76, E0508=1.
The unchanged `--all-targets` check emitted those 222 again plus 304 for the
lib-test compilation, totaling 526: E0204=56, E0277=62, E0599=30,
E0507=178, E0382=198, E0508=2. Deduplicating identical code/message/source
spans across the two compilations gives 304 distinct diagnostics (82 more
than lib-only). These are diagnostics, not necessarily distinct repair sites:
one `.copied()` can also trigger an iterator trait error.

The first full runtime captures exceeded the output limit. Both checks were
repeated with sources unchanged and a compact JSON parser; the counts above
are from those complete captures and agree with Cargo's error summaries.
No runtime repair or representation change preceded this evidence.

The first runtime repair was checked with `cargo check -p rum-runtime
--all-targets`: still failing, 152 emitted diagnostics, 80 distinct
(72 library / 80 lib-test). Emitted codes: E0204=12, E0277=6, E0599=4,
E0382=59, E0507=71. Removing `Copy` from structs/enums exposed further
transitive derives and moves; `.copied()` failures had also hidden later
body checks. This is additional observed fallout, so the initial 222/304
counts are not a complete migration estimate. A second repair follows;
the brief's two-failed-repairs stop-loss remains active.

The second runtime repair still failed: **5 library errors, 5 lib-test
errors (10 emitted, 5 distinct), all E0507**. The two repair checks therefore
failed for the same non-`Copy` borrowed-move class. **STOP-LOSS TRIGGERED.**
No further Rust source edits followed this result. The required verification
commands were then run on that stopped tree.

This is **not a complete per-crate fallout count**. Library errors observed
before repair total 223 across the two affected libraries reached (core 1,
runtime 222), with syntax 0. That is not a workspace total. Runtime's 304
initial distinct all-targets diagnostics include test-only fallout, but its
later checks expose additional compiler barriers. Residual counts below
are observations at different states, not disjoint sets to sum.

A live constructor search clarifies the brief's 179 figure: the four public
constructor names occur on **179 matching source lines, containing 180
expressions**. `crates/rum-runtime/src/runtime/options/core.rs:12` contains
two `Symbol::system` expressions. Neither the earlier report's 260 direct
constructor expressions nor these 179 lines/180 expressions measures
non-`Copy` compiler fallout.

Error classification covers every collected runtime diagnostic after
deduplicating exact code/message/source spans between lib and lib-test.
The single core diagnostic is an additional shared-reference dereference
move, E0507 at `crates/rum-core/src/expr.rs:134`. Representative line numbers
are those emitted at the relevant check, before later edits.

| Error kind | Initial runtime (distinct) | After repair 1 (distinct) | After repair 2 (distinct) | Representative compiler location |
|---|---:|---:|---:|---|
| Containing struct/enum cannot derive Copy (E0204) | 28 | 6 | 0 | `crates/rum-runtime/src/runtime/call_preparation.rs:7` |
| Option::copied Copy bound (E0277) | 13 | 1 | 0 | `crates/rum-runtime/src/runtime/call_preparation.rs:296` |
| Iterator/consumer Copy bound (E0277) | 24 | 3 | 0 | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:284` |
| Copied iterator methods unavailable (E0599) | 20 | 3 | 0 | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:285` |
| Dereference move through shared reference (E0507) | 55 | 28 | 5 | `crates/rum-runtime/src/runtime/attributes.rs:140` |
| Use/borrow after move (E0382) | 118 | 25 | 0 | `crates/rum-runtime/src/runtime/definitions/application/down_values.rs:165` |
| Move captured value out of reusable closure (E0507) | 3 | 0 | 0 | `crates/rum-runtime/src/runtime/definitions/application/up_values.rs:37` |
| Move out of Arc-held frame field (E0507) | 11 | 0 | 0 | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:285` |
| Move a field behind a shared reference (E0507) | 17 | 6 | 0 | `crates/rum-runtime/src/runtime/evaluation_cache/profile/report.rs:140` |
| Partial struct/field move (E0382) | 4 | 2 | 0 | `crates/rum-runtime/src/runtime/messages/message_catalog.rs:91` |
| Move out of indexed map/vector (E0507) | 10 | 3 | 0 | `crates/rum-runtime/src/runtime/scoping/lexical/block.rs:339` |
| Move out of indexed slice (E0508) | 1 | 0 | 0 | `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:197` |
| Pattern binding moves Option payload (E0382) | 0 | 3 | 0 | `crates/rum-runtime/src/runtime/call_preparation.rs:309` |
| Array / `[T; N]` repeat initialization requiring Copy | 0 observed | 0 observed | 0 observed | None reached |
| Total runtime diagnostics per observation | 304 | 80 | 5 | Do not sum across observations |

The E0277 categories include both the direct `.copied()` constraint and
consumer bounds induced by that iterator. E0599 diagnostics are cascading
method-resolution failures, not separate ownership operations. The
array-repeat row is zero only in the code reached before the stop; no claim
is made about downstream crates.

### 2. What changed

**74 Rust files plus this report**, all left uncommitted. This is a partial
Stage 1 repair, **not the Stage 2 Arc prototype**. `Symbol` still stores the
original dense ID and `&'static SymbolText`; global interning/leaking,
equality/hash/order, serde, and memory accounting have not been replaced.
`heads.rs` was not changed.

The edits replace invalid symbol copies with borrows or symbol clones,
remove invalid `Copy` derives from containing types, and preserve
continuation ownership. Profiling guard inputs borrow symbols so their
clones stay inside the existing enabled paths. No enclosing `Expr` was
cloned as a non-`Copy` repair. Tests received ownership repairs, with no
assertions or expected results weakened. These intentions are not a claim
of verified semantics or performance: the tree does not compile.

All changed Rust files have compiler diagnostics or compiler parameter notes
forcing their edits. There is no new `unsafe`. A before/after SHA-256
comparison of the scripts directory found no script changes. No session
interner, reclamation, canonical-pool redesign, or ledger accounting was
attempted.

Line ranges below refer to the stopped working tree. Error tags include
diagnostics that forced changes through a referenced parameter.

| File (runtime paths relative to `crates/rum-runtime/src/runtime/`) | Changed lines | Error kind(s) forcing the change | Why |
|---|---|---|---|
| `crates/rum-core/src/expr.rs` | 35, 134 | Stage 1, E0507 | Remove Symbol Copy; clone the canonical-map value. |
| `attributes.rs` | 140 | E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `call_preparation.rs` | 7, 15, 19–20, 23–24, 296–298, 300–301 | E0204, E0277, E0382 | Borrow metadata accessors; retain symbol fields without moves. |
| `definitions/application/down_values.rs` | 151, 166, 238, 267, 285, 354, 383 | E0382 | Borrow profile inputs or clone symbols retained by application state. |
| `definitions/application/sub_values.rs` | 98, 127, 199, 228, 247, 318, 347 | E0382 | Borrow profile inputs or clone symbols retained by application state. |
| `definitions/application/up_values.rs` | 37, 69, 270, 315 | E0507 | Borrow profile inputs or clone symbols retained by application state. |
| `definitions/definition_analysis_helpers.rs` | 261, 284, 312, 380, 386 | E0277, E0507, E0599 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/definition_catalog/entries.rs` | 88 | E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/definition_clear.rs` | 213 | E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/definition_lhs_preparation/prepared_store.rs` | 164, 231, 247, 253, 268, 293, 330, 357, 394, 412, 422 | E0277, E0382, E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/definition_tags.rs` | 64, 285 | E0382, E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/definition_updates.rs` | 544, 566, 595, 601, 624 | E0382, E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/definition_updates/resume_frames.rs` | 20 | E0507 | Clone symbol fields from shared frames; preserve resume/abandon flow. |
| `definitions/incremental_append.rs` | 187, 197 | E0382, E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/part_assignment.rs` | 29, 150 | E0277, E0382 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/part_assignment/resume_frames.rs` | 18 | E0507 | Clone symbol fields from shared frames; preserve resume/abandon flow. |
| `definitions/unique_symbols.rs` | 22, 35–36, 60, 62–63 | E0277, E0382, E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `definitions/value_table_assignment.rs` | 132, 144, 179 | E0382, E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `eval_core/stability.rs` | 368 | E0507 | Clone the symbol for the owned cache key. |
| `eval_core/symbol.rs` | 97 | E0382 | Clone the guard expression's symbol while retaining continuation ownership. |
| `eval_dispatch/mod.rs` | 46–47, 55, 196 | E0277, E0382 | Borrow metadata and builtin symbols across dispatch/profile reads. |
| `evaluation_cache/mod.rs` | 21, 29 | E0204 | Remove Copy from symbol-containing types and their enclosing keys/frames. |
| `evaluation_cache/profile/mod.rs` | 63, 253, 262, 271, 287, 306 | E0204, E0382, E0507 | Remove key Copy; borrow invalidation sources and clone retained keys. |
| `evaluation_cache/profile/report.rs` | 60, 100, 129, 140–141, 144–145, 182–183, 225–226, 256, 296, 333, 366, 404, 442 | E0507 | Clone non-Copy report keys/symbols from borrowed tables. |
| `evaluation_cache/profile/source.rs` | 7, 48, 55, 122, 169, 174 | E0204, E0277, E0382, E0507 | Remove frame Copy; borrow profiling inputs and clone stored symbols. |
| `evaluation_cache/stability.rs` | 107, 115, 119, 152, 158, 169, 192, 220 | E0382, E0507 | Borrow profiling sources; clone symbol cache-lookup keys. |
| `evaluation_cache/tests.rs` | 99, 109, 149, 252 | E0382 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `evaluation_cache/tests/association_lvalue.rs` | 47, 55, 70, 142, 181, 231, 251, 267, 282, 297, 399, 414 | E0382 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `evaluation_cache/tests/association_stability_tail.rs` | 161, 207, 214 | E0382, E0507 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `kernel_data/association_data.rs` | 869, 912 | E0382 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `kernel_data/association_data/resume_frames.rs` | 19 | E0507 | Clone symbol fields from shared frames; preserve resume/abandon flow. |
| `kernel_data/control_data/branching_loops/iteration_aggregate.rs` | 148, 221 | E0507 | Clone symbol ownership across maps, lexical bindings and continuation state. |
| `kernel_data/control_data/branching_loops/iteration/driver.rs` | 126, 185, 285, 351, 390, 409, 415, 471, 509, 525, 536, 545 | E0382, E0507 | Clone symbol ownership across maps, lexical bindings and continuation state. |
| `kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs` | 137, 149, 217 | E0507 | Clone symbol fields from shared frames; preserve resume/abandon flow. |
| `kernel_data/rewrite_data/suspended_state.rs` | 191 | E0507 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `messages/message_catalog.rs` | 89 | E0382 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `model/definitions.rs` | 209 | E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `namespace_tools/names.rs` | 278 | E0277 | Replace symbol iterator copied with cloned. |
| `options/builtins.rs` | 260, 269 | E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `package_tools.rs` | 419 | E0277, E0599 | Replace symbol iterator copied with cloned. |
| `pattern_matching_interpreted.rs` | 509 | E0507 | Clone the named-match symbol from its shared continuation frame. |
| `pattern_matching/core.rs` | 269 | E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `pattern_matching/core/resume_frames.rs` | 426 | E0507 | Clone symbol fields from shared frames; preserve resume/abandon flow. |
| `profiling/mod.rs` | 114, 124, 136, 143, 149, 166, 172, 178, 185, 193, 208, 219, 241, 250, 269, 277, 296, 304, 312, 333, 345, 359, 374, 395, 411, 452, 458, 510, 530 | E0204 | Remove Copy from symbol-containing types and their enclosing keys/frames. |
| `profiling/presence.rs` | 23, 27, 84, 89, 91, 95, 97–98, 151, 153–154, 184, 222, 240, 264, 267, 311, 339, 344, 362, 365, 370, 390, 398 | E0277, E0382, E0507 | Retain non-Copy profiling payloads with symbol/key clones. |
| `profiling/records.rs` | 243, 331, 340 | E0382 | Retain non-Copy profiling payloads with symbol/key clones. |
| `profiling/report_bindings.rs` | 12, 47 | E0507 | Clone non-Copy report keys/symbols from borrowed tables. |
| `profiling/report_presence_reuse.rs` | 12, 52–53, 103, 143–144, 197–198 | E0507 | Clone non-Copy report keys/symbols from borrowed tables. |
| `profiling/report_presence_sources.rs` | 12, 48, 85, 123, 167, 169–170, 223 | E0507 | Clone non-Copy report keys/symbols from borrowed tables. |
| `profiling/report_rebuild.rs` | 12, 46–47, 89, 123–124, 166, 196 | E0507 | Clone non-Copy report keys/symbols from borrowed tables. |
| `profiling/report_timing.rs` | 12, 46, 123, 180, 214, 254, 284, 338 | E0507 | Clone non-Copy report keys/symbols from borrowed tables. |
| `profiling/scopes.rs` | 79, 89 | E0382 | Borrow the profiling input; clone only after the existing enabled guard. |
| `profiling/timing.rs` | 86, 117 | E0382 | Retain non-Copy profiling payloads with symbol/key clones. |
| `resolution.rs` | 172 | E0382, E0507 | Clone symbols at compiler-reported borrowed moves or later reuse. |
| `scoping/gensym_gc.rs` | 24–25, 77, 225–226, 289–290 | E0277, E0507, E0599 | Clone symbol ownership across maps, lexical bindings and continuation state. |
| `scoping/lexical/block.rs` | 96, 127, 227, 238, 248, 257, 295, 299, 339–340, 363, 435, 451, 462, 471 | E0382, E0507 | Clone symbol ownership across maps, lexical bindings and continuation state. |
| `scoping/lexical/block/resume_frames.rs` | 28, 162 | E0507 | Clone symbol fields from shared frames; preserve resume/abandon flow. |
| `scoping/lexical/module.rs` | 197, 301, 329, 336, 365, 388 | E0277, E0382, E0507, E0508, E0599 | Clone symbol ownership across maps, lexical bindings and continuation state. |
| `scoping/lexical/with.rs` | 52, 112, 149 | E0507 | Clone symbol ownership across maps, lexical bindings and continuation state. |
| `scoping/packages.rs` | 20, 271, 278, 290, 298, 308, 396, 611, 613, 619, 683, 716 | E0277, E0382, E0507, E0599 | Clone symbol ownership across maps, lexical bindings and continuation state. |
| `session/session_api.rs` | 279, 293, 295, 301, 385, 388 | E0277, E0382, E0599 | Clone reused registration/snapshot symbols; two moves remain unresolved. |
| `tests/association_write_index.rs` | 23, 35, 121, 216 | E0382 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/association_write_recovery.rs` | 52, 89, 101 | E0382 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/definitions_rhs_guards_attribute_revision.rs` | 247 | E0382 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/mutation_operator_forms.rs` | 284, 588 | E0382 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/part_assignment.rs` | 75, 375, 438 | E0382 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/scoping_gensym_gc.rs` | 146, 178, 396, 432, 436, 448, 452, 459 | E0277, E0507, E0599 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/scoping_gensym_gc/reachability.rs` | 91, 174, 178, 334, 401, 438, 443, 450, 455, 462, 481, 494, 503, 513, 519 | E0277, E0382, E0599 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/scoping_gensym_gc/suspension_frames.rs` | 34, 41, 56, 62, 67 | E0277, E0507, E0599 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/sealed_packages.rs` | 95, 100, 224, 303 | E0277, E0507 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/sealed_snapshot.rs` | 18, 50, 263 | E0277, E0382, E0507 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/sealed_visibility.rs` | 26, 337 | E0382, E0507 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/suspension_abandonment.rs` | 475 | E0277, E0599 | Clone reused symbol fixtures; keep assertions and expected behavior. |
| `tests/valueq_stable_shortcut.rs` | 77 | E0507 | Clone reused symbol fixtures; keep assertions and expected behavior. |

The only documentation edit is this file's appended execution report,
tagged **report evidence**, from the `Execution report` heading to EOF.

Organization assessment: splitting existing modules would exceed this
brief's compiler-forced scope and mix an independent change into the spike.
Files in the 10k–20k warning range retain their existing domain boundaries.
The larger touched implementation files retain the following possible
future boundaries; none was split here:

| File | Future boundary, outside this spike |
|---|---|
| `crates/rum-core/src/expr.rs` | Symbol storage/traits vs numeric atoms vs shared expressions; separate its inline tests first. |
| `definitions/definition_lhs_preparation/prepared_store.rs` | Owner validation vs definition-store construction. |
| `definitions/definition_updates.rs` | Update preparation vs category-specific mutation. |
| `kernel_data/association_data.rs` | Association lookup/planning vs mutation/recovery. |
| `kernel_data/control_data/branching_loops/iteration/driver.rs` | Iterator driving vs dynamic-state capture/restore. |
| `package_tools.rs` | Package loading/resolution vs snapshot registration/relinking. |
| `pattern_matching_interpreted.rs` | Interpreted matching vs continuation frames. |
| `pattern_matching/core.rs` and `pattern_matching/core/resume_frames.rs` | Matcher execution/frame families by matching operation. |
| `scoping/lexical/module.rs` | Binding initialization vs activation/escape bookkeeping. |
| `scoping/packages.rs` | Declaration/binding collection vs sealed registration/rewrite. |
| `session/session_api.rs` | Session configuration/registration vs persistence APIs. |

The larger test files (`evaluation_cache/tests.rs`,
`tests/mutation_operator_forms.rs`, `tests/part_assignment.rs`,
`tests/suspension_abandonment.rs`) remain grouped by their current behavior
families. A future split should follow cache proof, operator mutation, Part
assignment, and abandonment scenario families respectively. No touched file
exceeds 35k characters.

### 3. Verification — failed, verbatim output

Checks were bounded to 120 seconds and offline. Warm checks observed 0.30s
for the repaired core library and 2.30s for syntax; runtime checks finished
within the initial one-second tool yield plus the following output poll.
The 120-second bound leaves substantial headroom over observed compile
times; neither mandatory command timed out.

`cargo fmt --check` exited **1**. Its first capture contained terminal color
escapes, so the identical command was repeated with `TERM=dumb NO_COLOR=1
CARGO_TERM_COLOR=never` for a plain-text capture. Removing only color controls
from the first output yields exactly the following text. No source edit
occurred between captures.

Command actually executed:

```text
TERM=dumb NO_COLOR=1 CARGO_TERM_COLOR=never CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

Verbatim plain-text output:

```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/definition_tags.rs:282:
         outcome: Result<Expr, EvalError>,
     ) -> crate::runtime::suspension::FrameAction {
         let resumed = match outcome {
-            Ok(rhs) => session.prepare_tag_set_assignment(self.tag.clone(), self.args.clone(), rhs, false),
+            Ok(rhs) => {
+                session.prepare_tag_set_assignment(self.tag.clone(), self.args.clone(), rhs, false)
+            }
             Err(error) => Err(error),
         };
         crate::runtime::suspension::FrameAction::complete(resumed)
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:171:
         Expr::Call {
             head: nested_head, ..
         } => match nested_head.as_ref() {
-            Expr::Atom(Atom::Symbol(symbol)) => CallStabilityHeadShape::CallHeadSymbol(symbol.clone()),
+            Expr::Atom(Atom::Symbol(symbol)) => {
+                CallStabilityHeadShape::CallHeadSymbol(symbol.clone())
+            }
             _ => CallStabilityHeadShape::CallOther,
         },
     }
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:104:
         record_evaluation_cache_counter(&SYMBOL_INVALIDATION_COUNT, "symbol_invalidations");
         let source = current_evaluation_profile_source_symbol();
         let location = Location::caller();
-        record_invalidation_site("symbol_invalidation", source.as_ref(), Some(symbol), location);
+        record_invalidation_site(
+            "symbol_invalidation",
+            source.as_ref(),
+            Some(symbol),
+            location,
+        );
         if let Some(source) = source {
             record_symbol_profile_counter("symbol_invalidations_by_source", &source);
         }
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:149:
         };
         let already_dirty = dirty_lane & bit != 0;
         let source = current_evaluation_profile_source_symbol();
-        record_invalidation_site("stability_dirty_write", source.as_ref(), Some(symbol), location);
+        record_invalidation_site(
+            "stability_dirty_write",
+            source.as_ref(),
+            Some(symbol),
+            location,
+        );
         if let Some(source) = source {
             record_symbol_profile_counter("stability_dirty_writes_by_source", &source);
             if already_dirty {
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:189:
                 "dirty_bloom_saturations",
             );
             let source = current_evaluation_profile_source_symbol();
-            record_invalidation_site("dirty_bloom_saturation", source.as_ref(), Some(symbol), location);
+            record_invalidation_site(
+                "dirty_bloom_saturation",
+                source.as_ref(),
+                Some(symbol),
+                location,
+            );
             if let Some(source) = source {
                 record_symbol_profile_counter("dirty_bloom_saturations_by_source", &source);
             }
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/tests/association_stability_tail.rs:204:
         vec![Expr::small_integer(2)],
     );
     let expr = association(vec![
-        rule(early_key.clone(), Expr::Atom(Atom::Symbol(unstable_symbol.clone()))),
+        rule(
+            early_key.clone(),
+            Expr::Atom(Atom::Symbol(unstable_symbol.clone())),
+        ),
         rule(Expr::string("later"), later_value),
     ]);
 
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:96:
 fn session_with_stable_identity_association(name: &str, identity: u64) -> (Session, Symbol) {
     let mut session = Session::default();
     let target = Symbol::with_identity(GLOBAL_CONTEXT, name, identity);
-    session.definitions.entry(target.clone()).or_default().own_value =
-        Some(association_with_integer_keys([1]));
+    session
+        .definitions
+        .entry(target.clone())
+        .or_default()
+        .own_value = Some(association_with_integer_keys([1]));
     session.bump_evaluation_epoch_for_symbol(&target);
     seed_association_stability_proof(&mut session, &target);
     (session, target)
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:106:
 fn session_with_stable_identity_list(name: &str, identity: u64) -> (Session, Symbol) {
     let mut session = Session::default();
     let target = Symbol::with_identity(GLOBAL_CONTEXT, name, identity);
-    session.definitions.entry(target.clone()).or_default().own_value =
-        Some(Expr::call(system_expr("List"), Vec::new()));
+    session
+        .definitions
+        .entry(target.clone())
+        .or_default()
+        .own_value = Some(Expr::call(system_expr("List"), Vec::new()));
     session.bump_evaluation_epoch_for_symbol(&target);
 
     let current = session
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:146:
         self: std::sync::Arc<Self>,
         context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
     ) -> crate::runtime::suspension::FrameAbandonAction {
-        context.restore_loop_iterator_scope(self.symbol.clone(), self.saved_state.0.as_ref().clone());
+        context
+            .restore_loop_iterator_scope(self.symbol.clone(), self.saved_state.0.as_ref().clone());
         crate::runtime::suspension::FrameAbandonAction::Discard
     }
 }
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:542:
 
         match saved_state.syntax_information {
             Some(syntax_information) => {
-                self.syntax_information.insert(symbol.clone(), syntax_information);
+                self.syntax_information
+                    .insert(symbol.clone(), syntax_information);
             }
             None => {
                 self.syntax_information.remove(&symbol);
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/messages/message_catalog.rs:86:
     ) {
         self.note_active_modules_escaped();
         if !self.message_definitions.contains_key(&key) {
-            *self.message_symbol_counts.entry(key.symbol.clone()).or_insert(0) += 1;
+            *self
+                .message_symbol_counts
+                .entry(key.symbol.clone())
+                .or_insert(0) += 1;
         }
         let definition = self.message_definitions.entry(key).or_default();
 
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/profiling/presence.rs:367:
             Expr::Call {
                 head: nested_head, ..
             } => match nested_head.as_ref() {
-                Expr::Atom(Atom::Symbol(symbol)) => PresenceRootShape::CallHeadCallSymbol(symbol.clone()),
+                Expr::Atom(Atom::Symbol(symbol)) => {
+                    PresenceRootShape::CallHeadCallSymbol(symbol.clone())
+                }
                 _ => PresenceRootShape::CallHeadCallOther,
             },
         },
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/scoping/lexical/block.rs:254:
 
             match state.syntax_information {
                 Some(syntax_information) => {
-                    self.syntax_information.insert(symbol.clone(), syntax_information);
+                    self.syntax_information
+                        .insert(symbol.clone(), syntax_information);
                 }
                 None => {
                     self.syntax_information.remove(&symbol);
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/scoping/lexical/block.rs:468:
 
         match saved_state.syntax_information {
             Some(syntax_information) => {
-                self.syntax_information.insert(symbol.clone(), syntax_information);
+                self.syntax_information
+                    .insert(symbol.clone(), syntax_information);
             }
             None => {
                 self.syntax_information.remove(&symbol);
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/association_write_index.rs:20:
     keys: impl IntoIterator<Item = i64>,
 ) -> Symbol {
     let target = Symbol::new(GLOBAL_CONTEXT, name);
-    session.definitions.entry(target.clone()).or_default().own_value =
-        Some(association_with_integer_keys(keys));
+    session
+        .definitions
+        .entry(target.clone())
+        .or_default()
+        .own_value = Some(association_with_integer_keys(keys));
     target
 }
 
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/association_write_index.rs:32:
     keys: impl IntoIterator<Item = i64>,
 ) -> Symbol {
     let target = Symbol::with_identity(GLOBAL_CONTEXT, name, identity);
-    session.definitions.entry(target.clone()).or_default().own_value =
-        Some(association_with_integer_keys(keys));
+    session
+        .definitions
+        .entry(target.clone())
+        .or_default()
+        .own_value = Some(association_with_integer_keys(keys));
     target
 }
 
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/part_assignment.rs:72:
     let (_, before) = seed_plain_list_path_stability(session, &target, &[0]);
     let lhs = Expr::call(
         system_expr("Part"),
-        vec![Expr::Atom(Atom::Symbol(target.clone())), Expr::small_integer(1)],
+        vec![
+            Expr::Atom(Atom::Symbol(target.clone())),
+            Expr::small_integer(1),
+        ],
     );
     let outcome = session
         .try_set_positional_part_lvalue(lhs, Expr::small_integer(9))
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/part_assignment.rs:372:
     dynamic_session
         .eval_text("dynamicPartOwner = {1, 2}")
         .unwrap();
-    let dynamic_depth = dynamic_session.push_dynamic_own_value_binding(dynamic_target.clone(), None);
+    let dynamic_depth =
+        dynamic_session.push_dynamic_own_value_binding(dynamic_target.clone(), None);
     assert_generic_part_assignment_replaces_root_args(&mut dynamic_session, dynamic_target);
     dynamic_session.truncate_dynamic_own_value_bindings(dynamic_depth);
 }
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/reachability.rs:447:
 
     session.collect_and_sweep_dead_gensyms(&[]);
 
-    assert_candidate_definition_count(&session, &[outer_candidate.clone(), inner_candidate.clone()], 2);
+    assert_candidate_definition_count(
+        &session,
+        &[outer_candidate.clone(), inner_candidate.clone()],
+        2,
+    );
 
     session.module_activations.pop();
     session.collect_and_sweep_dead_gensyms(&[]);
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/reachability.rs:454:
 
-    assert_candidate_definition_count(&session, &[outer_candidate.clone(), inner_candidate.clone()], 1);
+    assert_candidate_definition_count(
+        &session,
+        &[outer_candidate.clone(), inner_candidate.clone()],
+        1,
+    );
     assert!(session.gc_candidate_gensyms.contains(&outer_candidate));
     assert!(!session.gc_candidate_gensyms.contains(&inner_candidate));
 
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:15:
         .iter()
         .find(|unit| unit.package_context == context)
         .unwrap_or_else(|| panic!("missing sealed unit {context}"))
-        .private_index[name].clone()
+        .private_index[name]
+        .clone()
 }
 
 fn restore_snapshot(source: &Session) -> Session {
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/sealed_visibility.rs:23:
         .iter()
         .find(|unit| unit.package_context == "Pkg`")
         .expect("visibility probe should be registered")
-        .private_index["helper"].clone()
+        .private_index["helper"]
+        .clone()
 }
 
 fn eval_string_list(session: &mut Session, source: &str) -> Vec<String> {
```

`cargo test --workspace` exited **101** during compilation. **No tests ran;
there are no passing test counts to report.** This did not reach either of
the two assertions noted in TODO's consistency-review section, or any
semantic assertion about the old ID. No test contract was weakened.

Command actually executed:

```text
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout 120 -- cargo test --workspace
```

Verbatim output:

```text
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
error[E0507]: cannot move out of `*key` which is behind a shared reference
  --> crates/rum-runtime/src/runtime/profiling/report_presence_shapes.rs:12:34
   |
12 |             .map(|(key, stats)| (*key, *stats))
   |                                  ^^^^ move occurs because `*key` has type `runtime::profiling::PresenceShapeKey`, which does not implement the `Copy` trait
   |
help: consider cloning the value if the performance cost is acceptable
   |
12 -             .map(|(key, stats)| (*key, *stats))
12 +             .map(|(key, stats)| (key.clone(), *stats))
   |

error[E0507]: cannot move out of `*key` which is behind a shared reference
  --> crates/rum-runtime/src/runtime/profiling/report_presence_shapes.rs:82:34
   |
82 |             .map(|(key, stats)| (*key, *stats))
   |                                  ^^^^ move occurs because `*key` has type `runtime::profiling::PresenceContextShapeKey`, which does not implement the `Copy` trait
   |
help: consider cloning the value if the performance cost is acceptable
   |
82 -             .map(|(key, stats)| (*key, *stats))
82 +             .map(|(key, stats)| (key.clone(), *stats))
   |

error[E0507]: cannot move out of `*key` which is behind a shared reference
   --> crates/rum-runtime/src/runtime/profiling/report_presence_shapes.rs:118:34
    |
118 |             .map(|(key, stats)| (*key, *stats))
    |                                  ^^^^ move occurs because `*key` has type `runtime::profiling::PresenceArgSiteKey`, which does not implement the `Copy` trait
    |
help: consider cloning the value if the performance cost is acceptable
    |
118 -             .map(|(key, stats)| (*key, *stats))
118 +             .map(|(key, stats)| (key.clone(), *stats))
    |

error[E0507]: cannot move out of `*symbol` which is behind a shared reference
   --> crates/rum-runtime/src/runtime/session/session_api.rs:392:35
    |
392 |                 owned_units.entry(*symbol).or_insert(unit_index);
    |                                   ^^^^^^^ move occurs because `*symbol` has type `rum_core::Symbol`, which does not implement the `Copy` trait
    |
help: consider cloning the value if the performance cost is acceptable
    |
392 -                 owned_units.entry(*symbol).or_insert(unit_index);
392 +                 owned_units.entry(symbol.clone()).or_insert(unit_index);
    |

error[E0507]: cannot move out of `*symbol` which is behind a shared reference
   --> crates/rum-runtime/src/runtime/session/session_api.rs:404:77
    |
404 |                 if !persisted_set.contains(symbol) || !emitted_owned.insert(*symbol) {
    |                                                                             ^^^^^^^ move occurs because `*symbol` has type `rum_core::Symbol`, which does not implement the `Copy` trait
    |
help: consider cloning the value if the performance cost is acceptable
    |
404 -                 if !persisted_set.contains(symbol) || !emitted_owned.insert(*symbol) {
404 +                 if !persisted_set.contains(symbol) || !emitted_owned.insert(symbol.clone()) {
    |

For more information about this error, try `rustc --explain E0507`.
error: could not compile `rum-runtime` (lib) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `rum-runtime` (lib test) due to 5 previous errors
```

### 4. Stage 3 sizes and Stage 4 numbers — not attempted

Stage 1 stopped before the representation stage. No layout measurements
were made, including the base; the brief's prior 24/32-byte observations are
not substituted for measurements in this run.

| Type | Base bytes | Candidate bytes |
|---|---|---|
| `Symbol` | not measured | no prototype |
| `Atom` | not measured | no prototype |
| `Expr` | not measured | no prototype |
| `SharedExpr` | not measured | no prototype |

| Workload | Base instructions (3 runs) | Candidate instructions (3 runs) | Within-arm spread | Base/candidate allocations |
|---|---|---|---|---|
| Cold intern | not run | not run | unavailable | not run |
| Warm intern | not run | not run | unavailable | not run |
| Exact symbol-hot panel statements | not run | not run | unavailable | not run |
| Evaluation-dominated ordinary | not run | not run | unavailable | not run |

No `.rum` probe files were created, so there are no workload statements
to reproduce. No release build, `perf` count, multiplex-coverage result,
allocation run, or timing-panel A/B was credited. Stage 4's green-workspace
prerequisite was not met.

### 5. Verdict — evidence, no A1/A2 decision

The compiler evidence establishes that removing `Symbol: Copy` propagates
through library code, tests, containing keys/frames and profiling code, and
that constructor counts do not describe that work. It does **not** establish
the complete workspace repair cost: downstream crates and the core/syntax
test targets remain unmeasured.

There is no candidate-size or instruction/allocation evidence with which to
accept A1 or select the A2 fallback. No performance conclusion follows from
this stopped run.

The spike does **not** establish session ownership, reclamation on
`reset()`, escaped-value lifetime, or cross-session equality cost. No
session-import boundary or distinct-session text comparison was built or
measured. The original process-global leak remains.

### 6. Stop-loss / open questions

Stopped after the **second failed runtime repair**, under the explicit rule:
“if the same crate fails to repair twice for the same reason, STOP and report
with the count you have.” Both repair checks still reported E0507
non-`Copy` moves through shared references. The final five distinct sites
are:

| Remaining file | Lines | Unresolved move |
|---|---|---|
| `crates/rum-runtime/src/runtime/profiling/report_presence_shapes.rs` | 12, 82, 118 | `*key` from borrowed profiling maps |
| `crates/rum-runtime/src/runtime/session/session_api.rs` | 392, 404 | `*symbol` into snapshot ownership maps/sets |

The first file above remains unmodified because the stop-loss fired before
a third repair. The workspace is not green, formatting is not green, and
Stages 2–4 were not reached. The director must decide the next action; this
run does not presume authorization to bypass the stop-loss.
