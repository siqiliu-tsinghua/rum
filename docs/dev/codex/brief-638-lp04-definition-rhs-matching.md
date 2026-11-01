# Brief 638 — LP04: definition RHS, definition-rule and dispatch records

**Status:** implementation brief, staged. Director-written from the brief-624
design draft (`.temp/brief-624/drafts/LP04-definition-rhs-matching.md`); the
draft is raw material, this file is the authorization.
**Sequence:** LP01 (`3fd1885c`, contract and oracle), LP02 (`17576575`, the
record engine and queue), LP03 (`e924bb4f`, the evaluator, dispatch and
argument routes). LP03 established the mechanism this brief reuses; read its
final gate section (`.temp` copy of the engine is not authoritative — read
the tree).

**Amendment 1 (2026-11-01, before any source change):** the first dispatch
stopped at A1's preflight, correctly: two files this brief already requires
were missing from **Scope**. `profiling/tests/evaluator.rs` holds
`lp03_adopted_site_table_is_exact`, which asserts the exact 25-name set that
appending five keys to `PhaseKey::ALL` necessarily changes, and
`profiling/timing.rs` holds `open_phase`'s only production caller, whose call
shape item A changes. Both are authorized below, narrowly. The STOP was the
scope rule working; no source was touched.

**Amendment 2 (2026-11-01, director's census ruling R-638-1):** A1's shape
item A moved the rejection census from 1067 to 1066 (10/1021/35). One
redundant `.expect` rejection retired and two call-form rows re-spelled
inside RET002 (`mod.rs` 615->713, `timing.rs` 146->147); no semantic row and
no coverage claim moved. As `TEMPLATE.md` requires, the director ruled and
updated `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`
himself. That file is already modified in your tree: **do not edit it**, and
list it in your A2 `changes` with why `director census ruling R-638-1` so the
gate's tree check stays exact.

**A1 accepted (2026-11-01, `gate-638a1` GREEN).** Scope, fmt, check,
ownership, census (RULED at 1066), oracle, standing, fingerprints,
certificates, inventories and the workspace suite (4,033 passed, 0 failed,
61 ignored) all pass on the tree you left. Begin at **A2**; do not redo A1's
routes or its shape items.

**Amendment 4 (2026-11-01, A2's two STOPs ruled).** Both are accepted, and
both remedies are now in the brief:

1. **`&Session` starters.** The direct-default start at `down_values.rs:69`
   holds `definition` (an immutable borrow of `self.definitions` from `:36`)
   across the call, so a `&mut Session` starter cannot compile (E0502). The
   three-parameter remedy in `a2-readonly-starter-proposal.patch`
   (`runtime_path_timing_start_adopted`, its cold half, and `open_phase`) is
   authorized: the whole path only calls `logical_phase_start(&self, ..)` and
   mutates nothing. This is a signature-only change; do not restructure the
   borrows at the call site.
2. **The falsifier is rustfmt-clean now.** The `.temp` source was re-wrapped
   exactly as `a2-falsifier-format-proposal.patch` shows, the base
   calibration was re-run on the formatted bytes (same RED at
   `definition.rhs.total`, all 18 LP03 rows present), and the verbatim
   insertion now passes `cargo fmt --check`. Use the updated `.temp` copy.

**Amendment 5 (2026-11-01, director's census ruling R-638-2 and the dead
helpers).** A2's STOP is accepted, and both open items are ruled:

1. **The frontier is 1067 (10/1022/35), not 1066.** The authorized
   `&Session` receiver moved the frontier one level deeper.
   `session.logical_phase_start(..)` at `logical/mod.rs:713` was rejected as
   method syntax needing an unexpanded receiver reborrow (`&mut Session` ->
   `&Session`); with an exact `&Session` receiver it resolves, the walk
   enters `logical_phase_start` — its only caller — and stops at two RET002
   sites inside it: the `!lexical_profile_enabled()` operand (`:101:12-65`)
   and `Self::logical_phase_start_slow` (`:104:9-60`). Pairing: 1065 rows
   keep kind, file, callee and reason, one retires, two appear, zero
   base-only and zero candidate-only; definitions and roots stay 7707/7707.
   The callee's body was wholly behind the old boundary, so the analysed
   surface grew while the row count rose by one — no mark and no coverage
   claim moved. I reproduced the dump myself and it is byte-identical to your
   two. The census file is already at 1067 = 10/1022/35 with the ruling in
   its comment; include it in A2's `changes` and do not edit it.
2. **Delete the two now-unused helpers.** A2's adopted starts took the last
   callers of `runtime_profile_outcome_for_option_result` and
   `runtime_profile_outcome_for_internal_result`
   (`definitions/application/mod.rs:111` and `:126`). Delete exactly those
   two functions and nothing else in that file; new dead-code warnings are
   not acceptable. The pre-existing test-builtin warnings stay as they are.

After both: replay the inventories this source edit invalidates (contracts
first, then certificates and promoted rows), rerun the census, and complete
A2's acceptance. Then Stage B.

**Amendment 6 (2026-11-02, the gate's oracle red and the workspace duty).**
A2's remaining red is a derived inventory the A2 run list missed, and the
workspace duty moves to me.

1. **The constructor inventory is 175 (and 177).** The gate's `oracle` step
   runs `exact_types_and_constructor_claims_match_rustc`
   (`crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs`),
   which no A2 run covered. It fails at `total 175` against the recorded 174,
   by the same deeper walk as R-638-2: `logical_phase_start` was never entered
   while the `:713` call was rejected, and its early `return None`
   (`logical/mod.rs:102:20-102:24`, `Option<Option<Call>>::None`) is exactly
   one new live constructor context. Every other context is intact (the run
   prints 175 rows, one of them in `logical/mod.rs`). Update the two
   expectations (174 -> 175, 176 -> 177) and the comment with this reason,
   then run the test in both builds and disclose the change as a derived
   inventory replay. Do not hand-edit any other row of any inventory.
2. **The two full workspace suites are mine.** The sandbox cannot reach the
   systemd user bus, so `scripts/memory-scope.sh` fails before cargo. Do not
   attempt `cargo test --workspace` again, and never run it unscoped; the
   gate runs both builds outside the sandbox at 14G and eight threads. That
   is the A2 acceptance item that stopped the last two continuations, and it
   is now the director's.

After the oracle test is green in both builds, report A2 complete with the
run logs and **stop**: do not start Stage B in that continuation. Stage B is
dispatched only after I gate A2.

## Goal

Adopt the definition RHS, definition-rule and definition-dispatch phases —
routes `rhs-evaluate`, `rhs-total`, `definition-rule`, `direct-default`,
`down-matcher`, `sub-matcher` from
`.temp/brief-624/inventory/routes.tsv` — so that every logical definition
application produces exactly one record per route, opened at the legacy start
site and closed at the route's own consumer boundary, with `kind`, `symbol`
and `index` preserved for the rule route, across pause and resume.

Also close LP03's two shape items: `open_phase`'s `.expect` on a state its
only caller has already proved, and `begin_input_suspension`'s second
`.expect("just installed input owner")`.

Nothing about evaluation changes. Values, errors, traces, output and the
existing (legacy) profiler recorders stay exactly as they are. The disabled
path gains no work beyond the two relaxed loads it already pays.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md` (pitfalls; the standing inventory
  duty), `docs/dev/profiling.md`.
- `.temp/brief-624/DESIGN.md` — the contract. Read §"Contract" (lines 8–96)
  and §"Disabled-path gate and measurements for every implementation step"
  (265–328). The route tables are
  `.temp/brief-624/inventory/{routes,path-sites,expanded-paths}.tsv` and the
  definition sites are `.temp/brief-624/inventory/definition-sites.tsv`.
  Every site in them predates LP03, so line numbers may sit tens of lines
  late — locate by the phase string and the route rule, never by line alone.
  The line numbers **in this brief** were verified on the tree at `8a7eaae3`.
- The engine you build on:
  `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — `Call`,
  `PhaseKey` (25 variants), `open_phase`/`complete_phase` (the thread-local
  LIFO), `complete_logical_expr_defer_pause`, `pause_phase_at_finish`,
  `complete_phase_error_at_finish`, `finish_logical_*`, `PhaseCompletion`;
  `crates/rum-runtime/src/runtime/profiling/timing.rs` —
  `runtime_path_timing_start_adopted` and the
  `record_profile_runtime_path_timing_adopted*` family.
- The oracle that decides: `crates/rum-stdlib/tests/suspension_oracle/`
  (`fixtures/logical_profile/mod.rs` — `logical_counter_port`;
  `support/logical_profile.rs`; `manifest.rs`) and its inventories under
  `tests/suspension_oracle/inventory/`.
- My falsifier, already calibrated red on the base tree at
  `.temp/brief-638/falsifier/` (see **Falsifier** below), with the
  route-reachability probe that produced the programs it uses at
  `.temp/brief-638/falsifier/reach.log`.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` (the new phase
  keys, the rule opener/completer, the finish helpers and the `open_phase`
  signature; no new module)
- `crates/rum-runtime/src/runtime/profiling/logical/segments.rs` (the
  locally-closed-phase predicate, below)
- `crates/rum-runtime/src/runtime/definitions/application/rhs.rs`
- `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`
- `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs`
- `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs`
- `crates/rum-runtime/src/runtime/definitions/application/up_values.rs`
- `crates/rum-runtime/src/runtime/suspension/core.rs` (the shape item only)
- `crates/rum-runtime/src/runtime/profiling/tests/definitions.rs` (new; your
  unit tests)
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs` — the test
  `lp03_adopted_site_table_is_exact` only. Add exactly these five names to
  its `expected` set: `definition.rhs.total`, `definition.rhs.evaluate`,
  `down_values.direct_default_lookup`, `down_values.compiled_matcher`,
  `sub_values.compiled_matcher`. No other line of that file changes; no
  existing name, assertion or helper is removed, weakened or reformatted.
- `crates/rum-runtime/src/runtime/profiling/timing.rs` — the
  `runtime_path_timing_start_adopted` and `_slow` signatures (`&mut Session`
  → `&Session`, nothing else) and the single caller at `:149` for shape item
  A. Nothing else in that file.
- `crates/rum-runtime/src/runtime/definitions/application/mod.rs` — delete
  only `runtime_profile_outcome_for_option_result` (`:111`) and
  `runtime_profile_outcome_for_internal_result` (`:126`), which lost their
  last callers to A2's adopted starts (Amendment 5).
- `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` — **already
  modified by the director** under Amendments 2 and 5 (the expected frontier
  is now 1067 = 10/1022/35, ruling R-638-2). Do not edit it; include it in
  A2's `changes` as instructed there.
- `crates/rum-host/tests/lp04_definition_rows.rs` (new; my falsifier,
  inserted verbatim from the `.temp` copy)
- the standing inventories named under **Inventory replay** below.

Files and areas you MUST NOT touch:

- `crates/rum-runtime/src/runtime/definitions/application/default_values.rs`
  — it calls `evaluate_definition_application_rhs` at six sites (86, 96, 123,
  141, 219, 239) and those call sites stay as they are. The RHS routes open
  **inside** `evaluate_definition_application_rhs`, which is why this file
  needs no edit; if your design would require one, that is a STOP.
- the legacy recorder bodies (`definition_rule_timing_start`,
  `record_profile_definition_rule_timing`,
  `record_profile_definition_dispatch_*`, `record_profile_definition_rhs_eval`
  and its `_by_source` twin) and their call sites' arguments — they stay, byte
  for byte, at every site in this brief; `phase-sites.tsv` records them and
  the loss ledger's boundaries are derived from it;
- `fixtures/logical_profile/phase-sites.tsv` (no edit at all);
- the identity columns of `logical-profile-required.tsv` (first 16) and its
  frozen fingerprint `fnv1a64:b41ac6a5dbf0e9a4`; row count 1115, binding
  count 875, encoding-owned 240;
- `Expr`/`Symbol` representation, any public type or serde shape, codecs,
  guest encodings, semantics, error values, trace output, `crates/rum-core`;
- `AGENTS.md`, `TODO.md`, `Cargo.lock`, other briefs, earlier scratch.

## Shape rules (read twice; a departure is a STOP)

1. **One record per logical invocation.** The operation is the one named by
   the instrumented site and its input. A native reentry of the same
   operation is not a second invocation; a distinct recursive invocation, a
   distinct operand/iteration, or a later stage is.
2. **Start exactly at the legacy start site** of the same phase string, by
   converting that call to `runtime_path_timing_start_adopted`. Every site
   that starts one of the five adopted phase strings must be converted; a
   legacy start left unconverted for the same string is a STOP. **Every**
   start in the tree for each of these strings is listed in **Routes** below
   — the list is complete at `8a7eaae3`, verified by grep.
3. **Completion only at the route's named boundary**, with the route's own
   carrier: `Result<Expr, _>` for `rhs-evaluate`, the actual RHS result for
   `rhs-total`, `Result<Option<Expr>, _>` for `definition-rule` and
   `direct-default`, `Result<CompiledMatchOutcome, _>` for the two matcher
   routes. A record must never observe a carrier the route does not name.
4. **Pause = close with the input-required marker, at the completion site.**
   `complete_phase` already does this: given
   `RuntimeProfileOutcome::InputRequired` it calls `Call::finish_input_required()`,
   which marks the row (`epoch.input_required`) and closes the record. So at
   these sites **use `record_profile_runtime_path_timing_adopted`, not the
   `_defer_pause` variant** — the defer variant is LP03's shape for a site
   whose pause is owned by a later call, and here it would leave the LIFO
   entry dangling. The rule route does the same through
   `complete_definition_rule`'s `Err(InputRequired)` arm. No separate
   `pause_phase_at_finish` call belongs at these sites. The
   resumed half records **nothing** — it is the named loss the ledger
   measures. Do not invent a second record on the resumed arm, do not carry a
   `Call` in a suspension frame, and do not insert a `ProfileBoundary`; those
   are different mechanisms and they belong to a later brief.
5. **Finished exactly once.** A record either `finish`es once or is
   pause-finished once. `Drop` must never find an unfinished record on a
   completed program: after the run,
   `Session::logical_profile_test_assert_quiescent()` must pass and no row
   may have `open` or `abandoned` non-zero.
6. **No default-path cost.** All new work is behind
   `crate::runtime::profiling::lexical_profile_enabled()`, the same two
   relaxed loads the legacy starters pay; the slow path is `#[cold]
   #[inline(never)]`. No TLS lookup, allocation, `Mutex`, `Arc` clone or
   capture construction on the disabled path. No new `if token.is_some()`
   branches at call sites.
7. **Keys are aggregate, never identity.** Runtime routes use
   `AggregateKey::Runtime { phase, source }` with the phase string exactly as
   in `phase-sites.tsv`; the rule route uses
   `AggregateKey::Definition { kind, symbol, index }` with the **original**
   `kind`/`symbol`/`index` from the legacy `record_profile_definition_rule_timing`
   call beside it. Never key on a record id, a thread id or a guest value.
8. **Legacy recorders stay.** This brief adds the logical layer beside them.
   The legacy output, timing stack and `phase-sites.tsv` do not change.
9. **No semantic edit.** If a change would alter whether, when or with what
   value guest code runs, it is a STOP, not a cost fix.
10. **When the exact consumer boundary cannot be named, STOP and report the
    consumer you traced.** Do not approximate a completion site.
11. **No `.expect`/`unwrap` on a state the types allow**, in the code you
    touch. The two shape items below exist because LP03 left two behind.

## Engine additions (as code)

### 1. Five phase keys, appended

In `profiling/logical/mod.rs`, append to `PhaseKey` and to `PhaseKey::ALL`
(which becomes `[PhaseKey; 30]`). **Append only** — `from_name` is
index-based and reordering would silently remap every LP03 key:

```rust
    DefinitionRhsTotal,
    DefinitionRhsEvaluate,
    DownValuesDirectDefaultLookup,
    DownValuesCompiledMatcher,
    SubValuesCompiledMatcher,
```

`name()` arms, in the same order:

```rust
            PhaseKey::DefinitionRhsTotal => "definition.rhs.total",
            PhaseKey::DefinitionRhsEvaluate => "definition.rhs.evaluate",
            PhaseKey::DownValuesDirectDefaultLookup => "down_values.direct_default_lookup",
            PhaseKey::DownValuesCompiledMatcher => "down_values.compiled_matcher",
            PhaseKey::SubValuesCompiledMatcher => "sub_values.compiled_matcher",
```

`from_name` arms, indices 25–29, after the existing 24:

```rust
            "definition.rhs.total" => 25,
            "definition.rhs.evaluate" => 26,
            "down_values.direct_default_lookup" => 27,
            "down_values.compiled_matcher" => 28,
            "sub_values.compiled_matcher" => 29,
```

`site()` policies: `DefinitionRhsTotal` and `DefinitionRhsEvaluate` are
`Policy::Expr`; `DownValuesDirectDefaultLookup` is `Policy::OptionalExpr`;
`DownValuesCompiledMatcher` and `SubValuesCompiledMatcher` are
`Policy::Match` (the matcher's continuation slot is
`ContinuationState::MatchOutcome`, `suspension/typed/registry.rs:158`).

### 2. Two additions to the finish surface

`finish_logical_optional` already maps `Ok(Some) → Some`, `Ok(None) → None`,
`Err → error_outcome`, which is exactly the rule route's carrier — use it,
do not write a second copy of that mapping. What is missing is:

```rust
/// `rhs-total` in both overloads: a successful application total is `Ok`,
/// never `Some`, whatever the overload's carrier is.
pub(in crate::runtime) fn finish_logical_total<T>(
    call: Option<Call>,
    result: &Result<T, crate::runtime::EvalError>,
) {
    if let Some(call) = call {
        call.finish(match result {
            Ok(_) => Outcome::Ok,
            Err(error) => error_outcome(error),
        });
    }
}

/// The matcher routes' carrier. `CompiledMatchOutcome` has no guest slot of
/// its own; a match and a miss are both a completed logical call.
impl super::PhaseCompletion for crate::runtime::pattern_matching::CompiledMatchOutcome {
    fn timing_outcome(result: &Result<Self, crate::runtime::EvalError>)
        -> super::RuntimeProfileOutcome
    {
        phase_timing_outcome(result)
    }

    fn finish_logical(call: Option<Call>, result: &Result<Self, crate::runtime::EvalError>) {
        if let Some(call) = call {
            call.finish(match result {
                Ok(_) => Outcome::Ok,
                Err(error) => error_outcome(error),
            });
        }
    }
}
```

`CompiledMatchOutcome` is `runtime::pattern_matching::outcome`'s, re-exported
`pub(super)` from `runtime::pattern_matching` (`pattern_matching/mod.rs:10`),
so that path resolves from `profiling::logical`. If the trait cannot be
implemented there without widening visibility, STOP and report.

### 3. The locally-closed-phase predicate covers the rule record

`profiling/logical/segments.rs` already encodes "an adopted phase closes
locally on `InputRequired`; only an explicit LP02 boundary stays open across
the typed queue" in two places (`:131` in `restore`, `:199` in
`Segment::pause`), both by matching a `Runtime` key against `PhaseKey`. The
rule record is a `Definition` key, closes the same way, and **must be
skipped by both** — otherwise the pause cursor points at it (`:199`) and the
restore faults on a key it cannot resolve (`:131`). Replace both `matches!`
expressions with one predicate:

```rust
/// A record that closes locally at `InputRequired`, so the queue never
/// carries it: every adopted runtime phase, and every definition-rule record.
fn locally_closed(key: &AggregateKey) -> bool {
    matches!(key, AggregateKey::Runtime { phase, .. } if PhaseKey::from_name(phase).is_some())
        || matches!(key, AggregateKey::Definition { .. })
}
```

This is not cosmetic: without it the first paused definition application
faults the epoch, and every row it had recorded is retired.

### 4. The rule opener and its pause-aware completer

The rule route is keyed by `AggregateKey::Definition`, so it cannot ride
`PhaseKey` and the thread-local LIFO. It is a plain `Call` the caller holds:

```rust
impl crate::runtime::Session {
    #[inline]
    pub(in crate::runtime) fn logical_definition_rule_start(
        &self,
        kind: &'static str,
        symbol: crate::runtime::Symbol,
        index: usize,
    ) -> Option<Call> {
        if !crate::runtime::profiling::lexical_profile_enabled() {
            return None;
        }
        Self::logical_definition_rule_start_slow(self, kind, symbol, index)
    }

    #[cold]
    #[inline(never)]
    fn logical_definition_rule_start_slow(
        &self,
        kind: &'static str,
        symbol: crate::runtime::Symbol,
        index: usize,
    ) -> Option<Call> {
        let index = u32::try_from(index).ok()?;
        Call::start(
            self,
            AggregateKey::Definition { kind, symbol, index },
            Policy::OptionalExpr,
        )
    }
}

/// Completes the rule record with the application overload's actual result.
/// `Err(InputRequired)` is the pause: mark it and close, exactly as LP03 does.
pub(in crate::runtime) fn complete_definition_rule(
    call: Option<Call>,
    result: &Result<Option<Expr>, crate::runtime::EvalError>,
) {
    let Some(call) = call else {
        return;
    };
    match result {
        Err(crate::runtime::EvalError::InputRequired { .. }) => call.finish_input_required(),
        other => finish_logical_application(Some(call), other),
    }
}
```

`finish_input_required` is private to `logical/mod.rs` today; that is where
these live, so no visibility change is needed.

### 5. Shape item A — `open_phase` takes the key it was given

`profiling/logical/mod.rs:609`. Its only production caller,
`runtime_path_timing_start_adopted_slow` (`profiling/timing.rs:141`), resolves
the `PhaseKey` in its `if let` and then passes the phase string back down, so
`open_phase` re-derives it and `.expect`s on a state it cannot observe. Take
the key instead:

```rust
#[inline(always)]
pub(in crate::runtime) fn open_phase(
    session: &crate::runtime::Session,
    key: PhaseKey,
    site: AdoptedSite,
) {
    let Some(call) = session.logical_phase_start(key.name(), site.policy) else {
        return;
    };
    // unchanged from here
}
```

`&Session`, not `&mut`: the body only calls `logical_phase_start(&self, ..)`,
and A2's direct-default site holds an immutable borrow of `self.definitions`
across the start (`down_values.rs:36` .. `:70`), so a `&mut` starter cannot
compile there (E0502). The same relaxation applies to
`runtime_path_timing_start_adopted` and its cold half in `timing.rs`; nothing
on those paths mutates the session. Signature only — no call-site borrow
changes.

Callers: `runtime_path_timing_start_adopted_slow` (`timing.rs:149`) passes
the key it already holds instead of the phase string —
`if let Some(key) = PhaseKey::from_name(phase) { open_phase(session, key, key.site()) }`;
`open_logical_phase` (`:671`) becomes
`if let Some(key) = PhaseKey::from_name(phase) { open_phase(session, key, key.site()) }`.
No `.expect` remains, and the duplicated lookup is gone.

### 6. Shape item B — one `.expect`, bound once

`suspension/core.rs:505-531`. The same `Option` is unwrapped twice, two
statements apart:

```rust
        let pending = self
            .pending_suspension
            .as_mut()
            .expect("just installed input owner");
        pending.guest_form_environment = guest_form_environment;
        if crate::runtime::profiling::lexical_profile_enabled() {
            Self::enable_pending_profile(&mut pending.frames);
        }
```

`pending` must be dropped before the two `refresh_memory_owner` calls that
follow. `enable_pending_profile` keeps its `#[cold] #[inline(never)]` body.

## Routes

The line numbers are the current tree at `8a7eaae3`. The **legacy call beside
each site stays**; you convert it to the adopted form and, where the route
says so, add the logical completion.

| Route / phase key | Sites | Boundary |
|---|---|---|
| `definition.rhs.evaluate` (`Policy::Expr`) | `rhs.rs:18` (in `evaluate_definition_rhs`), `rhs.rs:63` (in `evaluate_definition_application_rhs`) | The existing legacy completion at `rhs.rs:21-24` and `rhs.rs:66-69`, on `rhs_result` — **before** `finish_definition_rhs_evaluation` absorbs `Return` (`:126`), so the record keeps the raw `Return`/`Throw`/`Error`. |
| `definition.rhs.total` (`Policy::Expr`) | `rhs.rs:14`, `rhs.rs:59` | The existing legacy completions: `rhs.rs:42-45` (non-application) and `rhs.rs:87-90` (application, success arm); on the application overload's suspend arm the legacy completion is at `rhs.rs:76-79`, where the result is `Err(InputRequired)` and the adopted completer closes the record with the input-required marker. A successful application total is `Ok` in **both** overloads, never `Some` — use `finish_logical_total`. |
| `definition-rule` (`Policy::OptionalExpr`, `AggregateKey::Definition`) | The eight `definition_rule_timing_start` sites: `down_values.rs:230`, `:333`, `:466`; `sub_values.rs:178`, `:318`; `up_values.rs:105`, `:185`, `:310` | Immediately after the existing `record_profile_definition_rule_timing(...)` call at each site, on that site's own `result` (`Result<Option<Expr>, EvalError>` before `map_err(Route::failure)`). `kind` is the string that call passes (`"down_value_rhs_eval"`, `"sub_value_rhs_eval"`, `"up_value_rhs_eval"`), and `symbol`/`index` are that call's arguments. |
| `down_values.direct_default_lookup` (`Policy::OptionalExpr`) | `down_values.rs:67-68` | The existing legacy completion at `down_values.rs:77-80`, on `direct_default` — **before** the `match` at `:81` and therefore before the fallback reads live definitions. `Ok(Some)` is `Some`, `Ok(None)` is `None`, never inferred from the eventual guest value. The `Err(InputRequired)` arm at `:84-93` needs no extra call: `:77-80` has already closed the record. |
| `down_values.compiled_matcher` (`Policy::Match`) | `down_values.rs:359-360` | The existing legacy completion at `down_values.rs:374-377`, on `match_result` — before the matcher-result consumer at `:379`. No extra call in the `Err(InputRequired)` arm at `:383-385`. |
| `sub_values.compiled_matcher` (`Policy::Match`) | `sub_values.rs:205-206` | The existing legacy completion at `sub_values.rs:220-222`, on `match_result` — before the consumer at `:226`. No extra call in the `Err(InputRequired)` arm at `:233-235`. |

Rules that are easy to get wrong, so they are named:

- **The resumed arms record nothing.** `finish_definition_rhs_evaluation`
  (`rhs.rs:118`) and the two matcher resume frames (`down_values.rs:499-512`,
  `sub_values.rs:350-358`) are untouched. If you find yourself adding a
  `logical_*` call there, you are building a different mechanism — STOP.
- **`definition.rhs.total` is not `Option`-shaped** even on the application
  overload, whose return type is `Result<Option<Expr>, _>`. `Ok(Some)` and
  `Ok(None)` are both `Outcome::Ok` for this route.
- **The rule record is not in the LIFO.** It is a `Call` the call site holds;
  the `definition.rhs.*` records opened *inside*
  `evaluate_definition_application_rhs` sit above it in the LIFO and are
  closed before the call returns, so on the pause path the finish order is
  inner-first: `definition.rhs.evaluate`, then `definition.rhs.total`, then
  the rule. Preserve that order. Note that `definition.rhs.total` is still
  open when the suspension is installed (`rhs.rs:72-73`) and the rule record
  is still open until the call returns — that is by design, and it is why
  **§3's `locally_closed` predicate must land in the same stage as the first
  paused site**. Ship §3 before or with A2's first pause, never after.
- **The compiled matcher runs after the fast match.** `down_values.rs:288-343`
  returns early on a fast-match outcome; only the fall-through at `:345-377`
  reaches the route. Do not move the record above the fast path.
- **`definition.rhs.finish` (`rhs.rs:122`) is not adopted.** Leave it legacy.
- **`definition_rhs_eval` and `definition_rhs_eval_by_source` are not new
  records.** They are counters on the same logical RHS operation; they ride
  the `definition.rhs.total` record. Do not open a second record for them.

## Inventory replay (standing duty; last step of the edit stage)

The rot guard hashes every `.rs/.py/.toml` under `crates/` (contract
fingerprints and certificate cells), and the LP01 inventories pin producer
files the same way. In this order, after the edits compile:

1. **Retention/contract replay** per `TEMPLATE.md`: regenerate the local
   contract fingerprints and the certificate cells by replaying
   `local_production_acceptance` in both builds, and reconcile the ownership
   rows. Disclose every changed row. **New `UnknownConstruct` rows for the
   new `Call`-holding locals and the new match arms are expected**; a new
   `UnknownHostPayload` row is a STOP.
2. **LP01 producer replay.** `cargo test -p rum-stdlib --test
   suspension_oracle -- logical_counter_port` fails naming
   `stale producer <file>` for every edited producer file. Recompute, never
   hand-edit, column 19 of `inventory/logical-profile-required.tsv` for every
   row whose producer file you edited: `fnv1a64` over `[file, file_text]` with
   LP01's own `fingerprint` function. Update column 17 (and the matching
   `logical-profile-bindings.tsv` column 3) only where a statement actually
   moved. The phase strings, the first 16 columns, the row count 1115, the
   binding count 875 and the frozen fingerprint
   `fnv1a64:b41ac6a5dbf0e9a4` must not change. Disclose every changed row.
3. **Loss-ledger replay under the amended bar.** Regenerate
   `inventory/logical-profile-losses.json` with the pin procedure, then
   extend `verify_losses.py` and make it print **all** of these **PASS**,
   keeping the existing receipt check:
   - **queued-unchanged** — for every `(mode, program, key)` present in both
     ledgers, the queued vector is identical;
   - **mode-confinement** — after the six approved producer-position moves,
     the `lexical` and `timing` rows are identical old to new;
   - **mass-never-up** — per `(mode, program)`, the L1 disagreement mass over
     its loss rows is `<=` the old ledger's; print lower/equal/higher counts
     and both totals;
   - **executed-receipts** — 4,375 receipts unchanged.

   The bar at `8a7eaae3` is queued 77,564 shared / 0 changed; mass 610 lower
   / 2,574 equal / 0 higher; 4,375 receipts; 714 vectors with zero offsets.
   Any increase under **mass-never-up** is a STOP. LP05–LP07 cells may move
   only under these checks. Never weaken one of them to reach green.
4. `cargo test -p rum-stdlib --test suspension_oracle` must be green in both
   the default and `--no-default-features` builds.

## Falsifier (director-written, calibrated red)

Insert `.temp/brief-638/falsifier/lp04_definition_rows.rs` verbatim at
`crates/rum-host/tests/lp04_definition_rows.rs`. It asserts, for a completed
program and for a program whose definition RHS pauses on `InputString` and
resumes: the adopted rows exist, `calls == ok + err`, `calls > 0`,
`abandoned = 0`, `open = 0`; the completed run's rows each have `ok > 0`; the
paused run's RHS and rule rows each have `err > 0`, and
`logical_profile_test_assert_quiescent()` holds.

Both probe programs were verified at HEAD to reach every route they assert —
the legacy recorder rows are in `.temp/brief-638/falsifier/reach.log`,
including the paused program's `definition-rule ... ok=0 err=1`.

Insert the `.temp/brief-638/falsifier/lp04_definition_rows.rs` file
byte-for-byte: as of Amendment 4 it is rustfmt-clean (one closure in
`kind`'s loop was re-wrapped; no assertion changed), so the verbatim copy
passes `cargo fmt --check`.

My calibration on the base tree (2026-11-01, `8a7eaae3`, 8 threads, log
`.temp/brief-638/falsifier/base-run-switch.log`, re-run on the formatted
bytes), **with the switch on**:

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp04_definition_rows
test lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs ... FAILED
panicked at crates/rum-host/tests/lp04_definition_rows.rs:118:36:
definition.rhs.total missing for `f[x_Integer /; x > 0] := x + 1; Default[f] = 3; f[2]; Default[f]`: [ ... 18 LP03 rows ... ]
```

The row dump is **not empty**: every LP03 route is present, and only the LP04
routes are missing. This is the calibration of record; an earlier run without
the switch also printed `[]`, which is what a broken engine looks like — as
LP03 learned, logical profiling only records when
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` is set. Every falsifier run below carries
the switch.

It must be green after LP04, and it is never weakened to reach green. If it
fails for a route you implemented but could not make reachable, that is a
STOP with the program and the consumer you traced.

## Verification — Stage A (the shape and its proofs)

**A1 — engine additions, the two RHS routes, and the two shape items.** Run
and paste verbatim:

```
cargo fmt --check
cargo test -p rum-runtime --lib profiling --offline  # your unit tests
cargo check --workspace --all-targets --offline
```

**A2 — the rule route at all eight sites, direct-default, both matcher
routes, and the falsifier.** Run and paste verbatim:

```
cargo fmt --check
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp04_definition_rows --offline
cargo test -p rum-stdlib --test suspension_oracle --offline -- logical_counter_port
cargo test -p rum-stdlib --test suspension_oracle --offline
cargo test --workspace --offline --no-fail-fast -- --skip lp03_adopted_phases_are_exact_in_completed_and_resumed_runs --skip lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs
cargo test --workspace --offline --no-default-features --no-fail-fast -- --skip lp03_adopted_phases_are_exact_in_completed_and_resumed_runs --skip lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs
```

The two `--skip` flags are the LP03 acceptance's rule: the host falsifiers are
switch-gated, so the plain suite skips them and they run separately with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` (this is exactly how LP03's workspace was
accepted). Do not add a global `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` to the
workspace run — 25 tests that assert the off path fail under it.

The **testing tier (user ruling 2026-11-01)** applies: while you are editing,
`cargo check --workspace --all-targets` plus the touched crate's `-p` tests
plus the named test is the gate. The two full workspace suites above run
**once each**, at A2's acceptance, not per change. The one full run before
your final report is the floor.

Your unit tests in `profiling/tests/definitions.rs` mirror
`profiling/tests/evaluator.rs`: a row finder over
`Session::logical_profile_test_rows()`, a completed down-value application, a
paused-and-resumed one, and one case each for `Default[...]`,
`sub_value_rhs_eval` and `up_value_rhs_eval`. Wire it exactly as LP03 wired
its own, at the foot of `profiling/logical/mod.rs`:

```rust
#[cfg(test)]
#[path = "../tests/definitions.rs"]
mod definitions;
```

Watch each new assertion fail before it passes and paste both.

The 48 brief-624 correctness programs must stay semantically green with
profiling off and on (`.temp/brief-624/correctness/` holds the fixtures and
`summary.tsv`; copy any runner into your scratch rather than writing to
brief-624). The oracle's `all` and `timing` modes are the authority for the
adopted cells; every adopted cell must be SAME and every previously adopted
cell must stay green.

## Verification — Stage B (measurement campaign)

Only after A2 is green. Read `.temp/brief-624/DESIGN.md:265-328` and run its
campaign with the director-supplied arms (base tree at `HEAD`, candidate at
your tree, one codegen unit, separate target dirs, offline, serial):

- one-codegen-unit instruction slopes for all 192 ordinary and 121 resume
  labels (`.temp/brief-624/performance/labels.json`), with the
  definition-heavy labels named explicitly in the report:
  `matcher-single-wide-small`, `matcher-single-wide-large`,
  `replacepart-wide-rule-list`, `appendto-list-incremental-large`,
  `appendto-list-incremental-small`, and each label's same-tier neighbours;
  `perf stat -x, -e cpu_core/instructions/u`, stdin `/dev/null`, pinned CPU;
- `scripts/core-language-microbench.sh` rum-vs-rum per-label `timings.tsv`
  on those labels and their same-tier neighbours, profiling OFF;
- the copy/allocation census on all 192 ordinary labels (brief-622 reducer);
- the 22 debug/release stack shapes (44 zero-residual fits).

The disabled path must not move: the existing gate is the bar. This brief
touches a hotter path than LP03 did — every definition application — so a
per-call increase on any ordinary label is a defect to attribute or report,
never noise. An external row (`__memcmp_*` and the like) is recorded, not
charged, but **the external class lapses on any non-zero rum-side row in the
same table** (LP03's ruling).

## Stop conditions and checkpointing

- STOP and report (do not improvise) if: a route's consumer boundary cannot
  be named; a record would need a carrier its route does not name; a change
  would alter guest semantics; a design needs an edit inside
  `default_values.rs`; the identity columns of the required inventory would
  move; mass-never-up fails; a new `UnknownHostPayload` row appears; the
  falsifier fails for a route you believe you implemented; or you cannot make
  a falsifier route reachable.
- Write `.temp/brief-638/PROGRESS.md` after every stop and every green stage:
  what is done and verified with raw paths, the exact next action, the tree's
  compile/test state, changed files, and the counters (adopted phases, rule
  rows, receipts, losses, ledger mass).
- Stop-loss is keyed on progress: two consecutive attempts that advance none
  of the brief's counters end the stage with a takeover checkpoint.

## Hard rules

- No version-control command in any form; leave the tree for the director.
- No network, no new dependency, no `unsafe`, no default-path cost.
- Keep every legacy recorder and `phase-sites.tsv` untouched.
- The two full workspace suites belong to the director's gate (Amendment 6):
  your sandbox cannot open the user scope, so do not run them at all. Timing,
  perf and callgrind runs are serial and alone with profiling off.
- Paste real output; say plainly what did not run. The report is
  `rum-report/1` at `.temp/brief-638/report-a1.json`, `report-a2.json`,
  `report-b.json`, and `python3 scripts/report.py check 638 --stage a1`
  (etc.) must pass before each stage's final message.
