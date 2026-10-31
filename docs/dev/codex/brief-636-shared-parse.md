# brief-636 — One immutable parse per test process: retention source, interfaces, typed discovery

**Shape: implementation.** Worker: kimi. The third of brief-633's three
implementation steps. brief-634 (artifact constructors) and brief-635
(profiler lease) have landed.

## Goal

At HEAD, every retention test and every `typed_slot_census` test parses the
whole workspace with `syn` and holds its own trees. Many tests also run
`Interfaces::new`. That work grows linearly with the libtest thread count:
- `rum-stdlib --lib` at 32 threads reaches the 14G cap (the director's
  calibration is below);
- brief-633 measured per-worker discovery at 4.7 GB for 4 workers and
  9.4 GB for 8.

syn trees are `!Send + !Sync`, so a `static` cannot hold them.

After this brief, each process builds three values once, immutably, and
`Sync`:
- the retention `Source`;
- its production `Interfaces`;
- the test-population source with its index.

The typed census builds its `Discovery` once in the same way.

A mutant is an explicit owned copy. Copy-on-write clones only the map that
changes. Every verdict, receipt, source coordinate and forced-red result
stays the same.

## Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, and brief-633's director gate
  (the end of `docs/dev/codex/brief-633-test-isolation-design.md`).
- Design evidence under `.temp/brief-633/`:
  - the draft `drafts/brief-shared-parse.md`;
  - the integrated tree `tree-final/` and its diff
    `logs/final/integrated.diff`;
  - the decomposition `inventory/e1-retention-decomp.tsv` and
    `inventory/e1-typed-decomp.tsv`;
  - the options `inventory/e2-options.tsv` and
    `inventory/e2-shared-option.tsv`;
  - the equivalence logs `logs/options/option-i-full-model-equivalence-r05.log`
    and `logs/typed-options/option-i-full-discovery-equivalence-r04.log`.
- Option (i), sharing the source alone, is **not** sufficient. The real
  eight-thread retention suite hit the 14G cap
  (`logs/final/retention-gate-r01.log`). This brief is option (iv).
- HEAD sites:
  - the retention `Source`: `crates/rum-stdlib/src/tests/retention_closure/model.rs:136-148`
    and `source.rs`;
  - `production()`: 27 call sites under `retention_closure/`;
  - `Interfaces::new`: `local/interface.rs`;
  - the typed `discover`: `crates/rum-stdlib/tests/typed_slot_census/discovery.rs`;
  - `Function.signature_ast`: `typed_slot_census/scan.rs:72`.

## Scope

- **MAY modify:** `crates/rum-stdlib/src/tests/retention_closure/**`,
  `crates/rum-stdlib/tests/typed_slot_census.rs` and
  `crates/rum-stdlib/tests/typed_slot_census/**`. Also the standing
  inventories, through their replays (`TEMPLATE.md` Scope).
- **May also update, under one rule:** `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv`.
  - A typed-slot X-row identity (`…#method9:…`, `…#reference57:…`) can move
    because this brief edits the function that contains it. Update that
    row's identity, keeping its X id, stage, population and disposition.
    List every before/after pair in `disclosures`.
  - A moved identity in a function this brief does not edit is a STOP.
- **MUST NOT:**
  - change production code;
  - change a retention census definition, reason or count
    (`census/tests.rs`), the pinned 1039-row frontier, `baseline`'s tuples,
    or the forced-red markers;
  - add a dependency or `unsafe` (that includes `unsafe impl Send/Sync`);
  - keep a `syn` or `proc_macro2` value in a static;
  - **shape code to keep an ordinal.** brief-633's tree-final added a
    behaviour-free type ascription to keep `#reference57`/`#method9`, and
    picked a macro destructuring to keep
    `legacy_macro_stays_opaque_in_all_four_views#reference7`. Write the code
    naturally, and move the identity in the inventory by the rule above.

## Target shape

The shape is fixed. A departure from it is a STOP.

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Shared<T>(Arc<T>);
impl<T> std::ops::Deref for Shared<T> { type Target = T; fn deref(&self) -> &T { &self.0 } }
impl<T: Clone> std::ops::DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T { Arc::make_mut(&mut self.0) }
}

#[derive(Clone, Default)]
pub(super) struct Source {
    pub functions: Shared<BTreeMap<Id, Vec<Function>>>,
    pub types: Shared<BTreeMap<Id, Vec<Nominal>>>,
    pub impls: Shared<Vec<(Id, String, Site, bool)>>,
    pub traits: Shared<BTreeMap<Id, Vec<Trait>>>,
    pub trait_impls: Shared<Vec<(Id, String, Site, bool)>>,
    pub constants: Shared<Vec<(Id, String, String)>>,
    pub files: Shared<BTreeMap<String, String>>,
    // every other heavy map or vector of Source uses Shared<_> as well
}

#[derive(Clone)]
pub(super) struct Function {
    pub sig: String,   // exact source slice, leading newlines/indentation kept
    pub body: String,  // so a local re-parse reproduces original line:column
    pub attrs: Vec<String>,
    pub impl_self: Option<String>,
    pub impl_generics: String,
    // identity, coordinates, substitutions and conditions: owned Sync data
}
impl Function {
    pub fn sig(&self) -> syn::Signature { /* parse self.sig locally */ }
    pub fn body(&self) -> syn::Block { /* parse self.body locally */ }
}

pub(super) fn production() -> &'static Source {
    static PRODUCTION: OnceLock<Source> = OnceLock::new();
    PRODUCTION.get_or_init(fresh_production)
}

impl Interfaces {
    pub fn shared_production() -> &'static Self {
        static PRODUCTION: OnceLock<Interfaces> = OnceLock::new();
        PRODUCTION.get_or_init(|| Self::new(source::production()))
    }
}

pub(super) struct SharedPopulation { pub source: Source, pub index: Interfaces }
pub(super) fn shared_population() -> &'static SharedPopulation { /* OnceLock, built once */ }

// typed_slot_census
type Discovery = (BTreeMap<String, String>, Facts, Report);
pub(super) fn production() -> &'static Discovery {
    static PRODUCTION: OnceLock<Discovery> = OnceLock::new();
    PRODUCTION.get_or_init(|| discover(&root()))
}
```

### Exact text and coordinates

These are calibrated in tree-final; keep them.

- **Slices.** Project each item from its file using span line and column.
  Do not use `byte_range()`: a nested group's range can be relative to its
  enclosing stream (tree-final saw `fn choose` at line 133 with range
  `160..184`).
- **Re-parsing.** Re-parse every stored string locally before any
  `tokens(...)` comparison. Never call `ToTokens` on a `String`: that makes
  a string-literal token.
- **Generics.** Clear generics by blanking their span in place. Add generics
  by inserting text right after the identifier.

### Sharing

- `Interfaces::new` projects its signature-only `Source` directly:
  - bodies are `{ }`;
  - `files` and `constants` start empty;
  - each function's `Self::syntax(f)` is computed once and reused for its
    fingerprint.
- Every fixture, every mutant and every changed interface still calls
  `Interfaces::new` on its own owned input.
- The typed census's `forced_red` reads `production()` and writes its scratch
  copy from that value's files. brief-634's `PrivateScratch` stays as it is.

## Must-not-change beside the cost bars

- **Equivalence, proved at stage 2 on complete dumps, not counts.**
  - Write two probes in `.temp/brief-636/probe/`, not landed. They build the
    HEAD-shaped fresh syn model and the shared model of the same tree.
  - Each probe dumps every function identity, census key and verdict, one
    per line, sorted, and the two dumps must be byte-identical (`cmp`).
  - Retention must give 7,660 functions and 50,584 keys, `Ok`. Typed must
    give 16,129 functions, 321,701 uses and 647 frames, `Ok`.
  - Save both dumps.
- **Receipts.** Every retention and typed-slot receipt is unchanged, except
  X-row identities moved under the Scope rule.
  - `source_baseline_matches_earned_receipt` prints the same baseline and
    `G15-C1` lines.
  - The legacy macro stays opaque in all four views.
- **Copy-on-write is a landed contract test.**
  - Clone `production()`, mutate one function's body in the clone, and
    assert three things:
    - the shared value's `functions` still points at the same `Arc` as
      before;
    - its text is unchanged;
    - the clone differs.
  - `&'static Source` cannot be mutated: the compiler enforces that.
- **Memory**, in each run's own 14G scope. This is a bound, never permission
  to change a receipt.
  - `cargo test -p rum-stdlib --lib --offline -- --test-threads=32` passes
    with `oom_kill=0`. Record its peak.
  - The typed target at 32 threads passes. Record its peak.
- **No measurement module** (`decomp`, `options_probe`) lands.

## Falsifiers

Each runs in its own `scripts/memory-scope.sh b636-<label>-<n> 14G` scope,
with `--offline` and `CARGO_BUILD_JOBS=8`. Copy HEAD to `.temp/brief-636/head/`
before your first edit, as brief-634 did.

1. **rum-stdlib lib at 32 threads.**
   - HEAD is killed at the 14G cap (the director's log is recorded below).
   - The change passes, with no cap kill.
   - Run it three times on the change. Bound: the director's HEAD
     measurement, recorded below.
2. **The retention suite at 8 threads and the typed target at 32 threads**:
   green on the change.
3. **Equivalence dumps**: byte-identical, as above.
4. **The copy-on-write contract test**: green on the change. It is red with
   `Shared::deref_mut` replaced by an in-place `Arc::get_mut().unwrap()` on
   a clone that is still shared: that panics. Save both logs.

## Stages and stops

1. **Shape.**
   - Build the shared `Source`, `Interfaces` and population, and the typed
     `Discovery`.
   - Then run `cargo fmt --check` and `cargo build --tests -p rum-stdlib
     --offline`, with no new warning.
2. **Equivalence.** Make the dumps and `cmp` them. STOP on any difference.
3. **Suites and memory.** Run falsifiers 1, 2 and 4.
4. **Replay and workspace.**
   - Replay the standing inventories, X-row moves included.
   - Then run `cargo test --workspace --no-fail-fast --offline --
     --test-threads=1`, bound 5400 s, in one 14G scope. The nine native
     ZeroMQ EPERM tests are skipped by name, as brief-634 lists them.

## Forbidden

- syn values in statics.
- Locks around discovery.
- Shared mutable overlays.
- Interior mutability in `Source`.
- Normalized-token substitutions that move spans.
- Ordinal-preserving code.
- Editing a receipt other than a moved X-row identity in an edited function.
- `#[ignore]` as a fix, retries, sleeps, raised or guessed bounds,
  `--test-threads=1` as a fix, merged tests, `serial_test` or any dependency.

## Report

- Write `.temp/brief-636/report.json`. Quote every `test result:` line, every
  verdict line, and both `cmp` results.
- Your final message is `python3 scripts/report.py summary 636`.

## Director calibration before dispatch (2026-10-31, HEAD `f38657d0`)

One run, serial, in its own 14G scope; the log is
`.temp/brief-636/calib-head.log`.

- `cargo test -p rum-stdlib --lib --offline -- --test-threads=32`, bounded
  at 3600 s: the test binary was killed at the cap.
  - `process didn't exit successfully: .../rum_stdlib-... --test-threads=32
    (signal: 9, SIGKILL: kill)`.
  - `# memory-scope: unit=b636-calib-head max=15032385536
    peak=15032385536 oom=13 oom_kill=1 rc=101`.
  - So HEAD reaches the 14G cap at 32 threads; falsifier 1's premise holds,
    and the worker's runs use this same scope, cap and bound.
