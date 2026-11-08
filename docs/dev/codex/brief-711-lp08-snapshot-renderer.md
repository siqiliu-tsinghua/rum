# Brief 711 — LP08 schema-v2 stage B: the exhaustive snapshot renderer

**Shape: implementation.** Brief 709 (design, accepted 2026-11-08) measured the
form and its six open questions are ruled in its Director gate; brief 710
installed stage A (the epoch-owned synchronous side index) at `e9ba6dd7`. This
brief lands **stage B only**: the renderer that publishes the additive
schema-v2 line from the native epoch fold, the side index, the Module-alias
projections and the eight fixed scalar defaults. The acceptance campaign
(stage C) stays separate.

The tree is `8ec1e9c1`. LP08's counter promotion landed at `b9b39763`; the
workspace is 4,169 passed / 0 failed / 61 ignored. `RumProfileSnapshot` still
returns `Null` after printing the legacy diagnostics
(`crates/rum-runtime/src/runtime/debug_tools/basic.rs:124-139`, producing the
target contract in `docs/dev/profiling.md:130-139`).

## Goal

Keep `RumProfileSnapshot` returning `Null` and keep every legacy diagnostic
byte-for-byte. Additionally emit exactly one new line per configured call:

```
rum-profile schema-v2 <LIST>
```

`<LIST>` is one expression in guest list syntax, built as text and **never
evaluated**, that publishes every logical key of `docs/dev/profiling.md:20-27`
exactly once, in a deterministic order, with the family's final vector,
explicit open/abandoned/cancelled accounting, a separate telemetry section and
a separate duration section whose basis is named. No top-N, no row cap, no
dedup of distinct keys, no guest evaluation, no public API change.

## Context pointers

- `docs/dev/profiling.md:20-27` (families), `:95-98` (bounded diagnostic
  storage), `:100-108` (lifecycle), `:113-128` (telemetry vs logical),
  `:130-139` (schema v2 target).
- The 709 rulings (binding): `.temp/brief-709/report-design.json`, its
  `DESIGN.md` §"Exact rendered form" and the design brief's Director gate
  (`docs/dev/codex/brief-709-lp08-schema-v2-publication-design.md:215-292`).
  Q1 duration basis, Q2 own-invocation qualification and header mode, Q3
  witness, Q4 transient budget, Q5 equal-clock invalidation, Q6 key encoding.
- The measured prototype: `.temp/brief-709/src/candidate/crates/rum-runtime/
  src/runtime/profiling/logical/publication.rs` and `logical/epoch/
  publication.rs`; the emitted examples in `.temp/brief-709/logs/final1-*.log`
  and `evidence/example-schema-v2.rum`. The prototype is **evidence of form,
  not a patch**: stage A (`e9ba6dd7`) moved the seams and the collector, and
  Q1/Q2/Q6 change the header, the duration source and the key text.
- Live machinery: `profiling/logical/epoch.rs` (Data, `event`, `event_rows`
  at `:165`, `retire`/`fault` at `:329`/`:355`, `current` at `:401`,
  `summary` at `:465`), `epoch/snapshot.rs:30` (`rows()` — the native fold
  that already adds the observing thread's running intervals via
  `segments::running`, `segments.rs:361`), `logical/record.rs:148` (`Counts`),
  `logical/totals.rs:21` (`Totals::report`), `logical/module.rs:42`
  (Module alias state), `logical/synchronous.rs` (`EventKey`/`EventValue`),
  `profiling/summary.rs:33` (`print_lexical_profile_snapshot`),
  `evaluation_cache/profile/mod.rs:143` (`evaluation_cache_profile_enabled`),
  `builtin_catalog/debug.rs:277` (the builtin registration).
- Standing oracles: `profiling/tests/logical_contract.rs`,
  `crates/rum-host/tests/logical_profile_lifecycle.rs`,
  `crates/rum-stdlib/tests/suspension_oracle/`.
- Instruments handed over: `.temp/brief-710/` (qualified base/candidate CLIs,
  Callgrind capture/reader scripts, the 120-label workload, the stack probe),
  `.temp/brief-709/` (family/caller crosswalk, five-mode runner, cost
  receipts). Their exclusions are restated below and are the instrument's,
  not restated per run.

## The published contract (the exact grammar)

The line is `rum-profile schema-v2 ` followed by one list expression. The
expression must parse with `rum_syntax::parse_expr`. Its top-level list is, in
this exact order:

```text
{"schema",2,"label",L,"epoch",E,"validity",V,"mode",M,
 "logical",{ROW,...},
 "telemetry",{"faults",F,"rows",{TELEMETRY_ROW,...}},
 "durations",{"basis","epoch-accrued+observing-thread-running","rows",{DURATION_ROW,...}}}
```

A failed initial allocation inserts `"reason","diagnostic-cap"` between
`"validity"` and `"mode"` (see below); no other form carries it.

- `L` is the label string quoted with `"`→`\"`, `\`→`\\`, newline→`\n`,
  CR→`\r`, tab→`\t`; every other character is verbatim. No raw newline may
  appear in the emitted line. The empty label is valid.
- `E` is the captured epoch's decimal id; `Null` when no epoch exists.
- `V` is `"valid"`, `"invalid"` or `"absent"`.
- `reason` appears **only** for a failed initial epoch allocation, as
  `"reason","diagnostic-cap"`, between `validity` and `mode`, and is sticky
  until a reset. An existing invalid epoch prints no reason here; its
  retirement diagnostic already carries it.
- `M` is the effective mode for the published families: the `+`-joined list,
  in this fixed order, of `lexical` (`lexical_profile_enabled()`),
  `builtin-timing` (`builtin_timing_enabled()`), `rule-rhs-timing`
  (`rule_rhs_timing_enabled()`), `definition-rule-timing`
  (`definition_rule_timing_enabled()`), `cache`
  (`evaluation_cache_profile_enabled()`), or `"none"` when the list is empty.
  Flags that only change legacy report content are not part of it. This is
  Q2's qualification: the own-invocation sentence holds where its family's
  gate is enabled, and the header says which gates are.
- `ROW` is `{"<key>","vector",{v,...},"open",o,"abandoned",a,"cancelled",c}`.
- `TELEMETRY_ROW` is `{"<key>",{v,...}}`; `DURATION_ROW` is `{"<key>",{v,...}}`.
- Every logical key appears exactly once in each of the three sections, in the
  same bytewise order of its encoded key text. An empty epoch prints `{}` for
  every section.

### Key encoding (Q6) and key spelling

Symbols are encoded **injectively from their components**, never from
`fully_qualified_name()`, `Symbol::Ord`, `interned_id`, addresses or threads:

```text
symbol(s) = "s(" ESC(s.context()) "," ESC(s.name()) "," (s.identity() decimal | "n") ")"
source(None) = "-"
source(Some(s)) = symbol(s)
ESC: '%'->"%25"  '|'->"%7C"  ','->"%2C"  '('->"%28"  ')'->"%29"  '#'->"%23"
```

The family key spellings (phase/kind/event/name/arity/index are closed-set
identifiers or decimal text; they need no escaping):

```text
runtime|{phase}|{source}
builtin|{symbol}|{source}|{arity}
definition-rule|{kind}|{symbol}|{index}
rule-rhs|{phase}|{source}
definition-dispatch|{kind}|{symbol}|{source}
counter|{name}
cache|{NAME}
cache-symbol|{event}|{symbol}
cache-source|{event}|{symbol}
```

Worked spellings the tests pin: `runtime|compound.item.suspend|-`;
`builtin|s(System,RumProfileSnapshot,n)|-|1`;
`definition-rule|down_value_rhs_eval|s(Global,witness,n)|0`;
`definition-dispatch|down_value|s(Global,witness,n)|-`;
`cache-symbol|builtin_applications|s(System,RumProfileSnapshot,n)`;
`cache-source|definition_rhs_eval_by_source|s(ctx,name,3)`.

### The union

Publish `N ∪ S ∪ A ∪ Z` exactly as the design defines it (709 `FAMILIES.md`
§"Exact key union"):

- **N** — every native epoch row from `Epoch::rows()` (`epoch/snapshot.rs:30`;
  this is the Q1 fold, including the observing thread's running intervals),
  with its logical vector, accounting and telemetry.
- **S** — every side-index row from stage A (`Epoch`'s `events` index), with
  the family's original vector; zero open/abandoned/cancelled.
- **A** — Module aliases, projected once from native
  `runtime|module.eval.total|<source>` rows: `counter|eval_module` sums calls
  and open/abandoned/cancelled across sources into its Z row, and each
  `Some(source)` also yields `cache-source|eval_module_by_source|<symbol>`
  with calls, accounting and `[nanos,self_nanos]`. No second writer.
- **Z** — the eight fixed LP01 scalar defaults, always present as zero rows
  (vector `{0}`, accounting zero, empty telemetry, durations `{0,0}`):
  `counter|definition_rhs_eval`, `counter|eval_module`,
  `cache|EVALUATE_ENTRY_COUNT`, `cache|DOWN_VALUE_SCAN_COUNT`,
  `cache|DOWN_VALUE_APPLICATION_COUNT`, `cache|SUB_VALUE_APPLICATION_COUNT`,
  `cache|BUILTIN_APPLICATION_COUNT`, `cache|EVAL_MODULE_ENTRY_COUNT`.

A real equal scalar replaces its Z default; defaults are never additional
rows. A **second owner for one encoded key** (N vs S, N vs A, S vs A) is an
instrumentation ownership collision: the receipt must be visibly invalid and
the epoch must be marked with the existing `Epoch::fault()` — never a panic,
never a silent overwrite, never a test-only assert. A deterministic winner is
not a substitute for the fault.

### Section contents

Logical vectors, fixed widths: runtime 7 (`calls, ok, err, some, none,
return_err, throw_err`); builtin 3 and definition-rule 3 (`calls, ok, err`);
rule-rhs 3 (`calls, changed, unchanged`); definition-dispatch 6 (`candidates,
prefilter_skips, fast_matches, fast_misses, matcher_matches, matcher_misses`);
counter/cache/cache-symbol/cache-source 1 (count). A native
`AggregateKey::RuleRhs` row has no production constructor today and its
`Counts` cannot fill the RHS vector honestly; if one is ever present, treat
it as an instrumentation ownership collision (visible fault), never map
`calls/ok/err` onto `calls/changed/unchanged`.

Telemetry vectors: native rows 14 (`input_required, pauses, segments`, then
the eleven `abandonment_reasons`, in the existing `Retire` index order); side
rows 1 (`input_required`); Z and A-only rows empty (`{}`). `faults` is
`Epoch::faults`.

Duration vectors: native rows and A source rows `[nanos, self_nanos]`; Z rows
`{0,0}`; every side row prints its four stored slots — runtime/source rows
`[nanos, self_nanos, 0, 0]`, rule-rhs and scalar rows `[nanos, 0, 0, 0]`,
dispatch rows `[prefilter_nanos, fast_match_nanos, compiled_matcher_nanos,
rhs_substitution_nanos]`, cache rows zero. The four side slots are fixed;
reserved slots are not logical fields. The basis token is the Q1 basis and
must be emitted exactly as written above.

### Validity and lifetime

- Absent: do not create an epoch. Print `epoch Null`, `validity absent`,
  empty sections; the mode is still stated.
- Failed initial allocation: the sticky failed id, `validity invalid`,
  `reason diagnostic-cap`, empty sections.
- Existing invalid epoch: its id, `validity invalid`, its retained rows and
  fault count. It cannot certify parity; do not repair or re-baseline it.
- Capture the epoch `Arc` once, build the copy, and only then read
  `is_valid()`/`faults`: a concurrent reset must yield an invalid receipt,
  never a valid header over a torn copy.
- The snapshot neither finishes nor restarts an open record; the snapshot's
  own entered builtin invocation stays open where its family is enabled
  (`open = 1`, zero terminal vector).

## Scope

MAY modify: a new private `profiling/logical/publication.rs` (or a private
`publication/` directory under `logical/`); `profiling/logical/mod.rs` (module
declaration only); `profiling/logical/epoch.rs` (un-gate the snapshot module,
add the non-creating publication-state accessor and a production side-row
accessor, keep every existing body); `profiling/logical/epoch/snapshot.rs`
and `logical/segments.rs` (compilation guard / visibility only, bodies
unchanged; items that stay test-only keep their own cfg so production gains
no dead code or warning); `profiling/summary.rs` (the one call, inside the existing
`lexical_configured` branch, before the legacy lines as measured);
`evaluation_cache/profile/mod.rs` (one `pub(in crate::runtime)` gate accessor
for the mode token only); `builtin_catalog/debug.rs` (the
`RumProfileSnapshot` description string only); `profiling/tests/mod.rs` plus
a new private test module; a new `crates/rum-host/tests/schema_v2_snapshot.rs`;
`docs/dev/profiling.md` and the regenerated `docs/reference/builtins.md`;
derived inventories under `crates/rum-stdlib/src/tests/rum-retention/` **only
as canonical replay output**.

MUST NOT modify: `debug_tools/basic.rs` (label/arity/Null semantics stay
exactly as they are); `logical/record.rs`, `totals.rs`, `index.rs`,
`segments.rs`, `synchronous.rs` and the seven stage-A recorder seams
(`profiling/timing.rs`, `profiling/counters.rs`,
`evaluation_cache/profile/source.rs`, and `profile/mod.rs` beyond the one
accessor); `PhaseKey` membership; native completion order; carriers; token,
record and queue layouts; promotion sets, loss dispositions or oracle
vectors; the LP01/LP02 observers' expectations; `Expr`/`Symbol`/`rum-core`;
public APIs, features, serde, manifests or dependencies; any hot-path gate or
branch; any `unsafe`. No re-baseline, no oracle weakening, no hand-edited
inventory.

## Target shape

```rust
// profiling/logical/publication.rs
pub(in crate::runtime) fn render(label: &str) -> String; // pure: no I/O, no lock held on return
pub(in crate::runtime) fn print(label: &str);           // eprintln!("rum-profile schema-v2 {line}")

// profiling/logical/epoch.rs
pub(in crate::runtime) fn publication_state() -> (Option<Arc<Epoch>>, Option<u64>); // never creates
impl Epoch {
    // production-visible: rows() today in epoch/snapshot.rs:30, events() the
    // side index's production accessor (the existing test accessor keeps its
    // cfg and its lease check).
}
```

Both accessors keep the existing `cfg(test/feature)` lease refusal — an
unregistered test thread sees nothing, exactly as `current()` does — and
production compiles the check out.

`summary.rs` keeps the existing gate: the call sits inside `if
lexical_configured`, before `eprintln!("rum-profile lexical snapshot
label={label}")`; the legacy lines' text, order and the once-flag are
unchanged. With no `RUM_PROFILE_*` configured, the branch is not reached and
the emitted bytes equal today's. The line is emitted whenever
`lexical_configured()` holds — the same branch that prints the legacy lexical
diagnostics — so a lexical-configured/deferred process prints `validity
absent` with `mode none` rather than silence.

## Invariants

- Locks: no interner access, string formatting, allocation beyond the epoch's
  own copy, or output while the epoch data lock is held. `rows()` and the
  side accessor each take and release their own lock; no cross-family
  atomicity is claimed.
- Q4 transient policy: the copy and the rendered text are bounded by the
  captured epoch's own storage (the 16 MiB diagnostic ceiling) and are **not**
  charged to the budget a second time. No new fallible-copy failure path.
- Determinism: for one epoch state the emitted text is identical across
  processes and across a thread handoff (epoch id aside). Sorting is over
  encoded key text, never `Symbol::Ord`, interned ids, addresses or threads.
- No truncation: more than 64 distinct keys publish every key. Equal keys
  aggregate once; distinct semantic identities never merge.
- Disabled/default path: profiling off adds no executed instruction, no
  stack byte and no new cold-load on any path that runs today.
- The guest outcome never changes: every mode still returns `Null`; an
  instrumentation fault never alters a guest value or resource outcome.

Forbidden shapes: deriving the key text from `Debug`/`Display` of an
`AggregateKey` and rewriting it; a global or thread-local cached line or
memoized union (every call observes its own captured epoch); charging the
copy to the budget or retiring on a copy allocation failure; omitting an
empty section instead of printing `{}`; `unwrap`/`expect`/`panic!`/
`unreachable!` on any state the types allow in production code — the
collision, cap and invalid paths are visible faults, not panics; sorting or
iterating a hash container to build the order.

## Gate tests and falsifiers

A private runtime test module, lease-held and serialised, that parses the
rendered line with `rum_syntax::parse_expr` and walks it — the emitted text,
not an intermediate, is the guest side of every comparison. The test must be
written so that an unparseable emission fails, not skips:

1. **Union and parity.** A two-thread resume witness, plus the minimal
   handoff program (`f[] := Module[{x=3}, (InputString["a"]; x)]`, started on
   A and resumed on B) that covers all five historically missing families
   (`cache`, `cache-source`, `cache-symbol`, `counter`, `runtime`): every key
   appears exactly once in each section, the family key count is reported per
   mode, the logical vectors and accounting equal the native observer
   field-for-field, and the comparison decodes the emitted key text back to
   components and matches the native enum fields, so the encoder is never its
   own oracle.
2. **Five modes.** Disabled, lexical, timing, cache, all-detail (the 709
   `run.py` flag sets): expected family presence per mode, the mode token
   exact, the own builtin row open only where builtin timing is enabled.
3. **Mid-interval durations (Q1).** Render while a native record is open on
   the observing thread; the duration row lies between the native fold read
   immediately before and after the render (record both reads); after closure
   the two are exactly equal.
4. **No truncation.** At least 96 distinct definition keys all present.
5. **Equal keys.** A repeated equal key is one row with the summed vector.
6. **Distinct identities.** Same text with identities 3 and 9 remain two rows.
7. **Adversarial keys.** Host-created context/name pairs with identical
   `fully_qualified_name()` concatenation stay distinct; `None` and
   `Some("","-")` stay distinct; escaping is exercised; the emitted line
   still parses.
8. **Determinism.** Two child processes with reversed interner insertion
   produce identical ordered key text; a thread-handoff witness matches.
9. **Labels.** No argument, string, nonstring and excess arity: legacy
   behaviour unchanged, the new line's label escaped, empty label valid.
10. **Lifecycle.** Nested reset (old/new epoch ids and rows), old-marker
    equal-clock fault, drop-before-summary, diagnostic-cap while a
    continuation is paused (invalid header, and the guest resume still
    completes), sticky initial-allocation failure, collision fault, absent
    epoch (no creation).

A new host test (`crates/rum-host/tests/schema_v2_snapshot.rs`) owns the guest
channel: `RumProfileSnapshot` returns `Null` in every mode; exactly one
schema-v2 line is emitted when lexical is configured and none when it is not;
the disabled-mode stderr carries no `rum-profile` byte (the gate reproduces
the pre-change bytes on the base tree); with the schema line removed, the
remaining legacy diagnostics are byte-identical to the base's; label and
arity behaviour are unchanged.

Mutants (install each in a scratch copy with a private target; the named test
goes red at 8 and 1 threads, the restored code green; quote every verdict
line):

1. `take(64)` on the logical rows → no-truncation red.
2. Sort by `Symbol` id/`Debug` instead of encoded text → determinism red.
3. Read a TLS physical map instead of the epoch → handoff/mode red.
4. Synthesize a side event equal to a native key → collision-fault red.
5. Drop one Z default row → union red.
6. Collapse the identity suffix to `n` always → distinct-identities red.
7. Finalize the snapshot's own record before rendering → own-open red.
8. Emit `validity valid` for an invalid epoch → cap/invalid red.
9. Hardcode the mode token → five-mode red.
10. Read `data.rows` directly instead of `Epoch::rows()` → mid-interval red.

## Disabled cost

The bar is brief 709/710's: at one stable absolute path, release, CGU=1,
jobs 8, offline/locked, two independent builds per arm; the Function source
rows (`try_apply_function`, `finish_function_application`,
`apply_status_function`, `evaluate_status_function_value`) and the resumed
frame row must match the stage-A base exactly in own/incoming vectors and
slopes; the queued F6 control is `try_apply_function` 49 instructions/call;
the Function stack is 18,304 bytes/level with intercept 12,392, zero residual
and the same eight absolute samples (four depths x two). Whole-workload
instruction draws are context only: use the calibrated ledger scope of Ruling
8 (the changed code's own disabled path plus the whole-workload total), never
a whole-process zero-tolerance all-function reducer (its base-vs-base null
fires on an identical binary). Persist raw output before parsing; Callgrind
serial and CPU-pinned.

**Must not change beside this bar:** two actual `InputString` pauses, prompt
and answer order, Function result 7, entered scopes, terminal outcomes, cache
entry ordering, sampled Return/Throw boundaries, token size, the native stack
limit, every phase name and vector. No cost repair by removing work,
delaying collection, duplicating bodies, lazy initialization, flattening
carriers, silent drops or debug-only inlining.

## Verification

One cargo command at a time, offline/locked, jobs 8, tests 8, plain inside
the dispatcher's 16G scope; mid-change `cargo check --all-targets` plus
`-p rum-runtime`, the full `cargo test --workspace` once at acceptance. Replay
every affected standing inventory canonically **before** the first acceptance
suite and quote the replay verdicts; install the canonical output, disclose
every changed row with its site, never hand-edit. A row that disappears,
changes status or becomes self-reaching is a STOP; position-only moves and
new rows naming the new construct are disclosed for the gate. A red is
reproduced at `--test-threads=1` before it is called a defect. `cargo fmt
--check` clean; focused suites at 8 and 1 threads; the seven switch-gated
names under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`; the five-mode LP01 parents
(88,795 records, `semantic_differences=0`, `byte_identical=true`); the host
lifecycle suite 6/6; the workspace once with the seven switch-gated names
skipped, no new warning, peak memory recorded. The nine native ZeroMQ tests
that a worker sandbox denies (`TcpListener::bind` → `Operation not permitted`)
are the known environment boundary: reproduce at 1 thread, report them as
that boundary, and never add a skip; the director runs the final workspace on
the host.

## Checkpointing

Keep `PROGRESS.md` with the exact next action and the receipts already saved;
leave a compiling checkpoint after the shape and after the proofs; preserve
failed logs. All scratch under `.temp/brief-711/`. No git.

## Deliverable

- The renderer, the accessors, the runtime model tests and the host test.
- `docs/dev/profiling.md` updated to this exact grammar, with Q2's
  own-invocation sentence qualified (the family gate decides, and the header
  states the effective mode) and Q1's basis named; `docs/reference/builtins.md`
  regenerated from the registry (`cargo run -p rum-stdlib --bin
  generate-builtin-reference`) after the description change, never hand-edited.
- The exact changed-line and ripple list; every test verdict line at 8 and 1
  threads; the ten mutant reds with restored greens; the two-build cost rows;
  all replay verdict lines; the emitted schema-v2 line for each of the five
  modes.
- `.temp/brief-711/report.json` in `rum-report/1`, plus
  `python3 -I scripts/report.py check 711` summary and every compliance
  verdict line.

## Hard rules

AGENTS and TEMPLATE apply in full. No vendor differential engine, no
re-baseline, no reclassification of a logical row as telemetry, no widening
of a family gate, no new public surface, no guest evaluation from the
renderer. A departure from the contract above, an unapproved family or an
unparseable emitted expression is a STOP: report the evidence and the minimum
scope change. The director owns the gate and every commit.
