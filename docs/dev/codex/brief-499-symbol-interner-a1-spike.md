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
