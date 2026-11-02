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
campaign with the director-supplied arms (base = the director's clean export
at `.temp/director/gate-state/worktree`, whose `crates/` equals `e924bb4f`
and which you already verified against all 1537 snapshotted source files;
`HEAD` now carries the LP04 candidate, so do not use it as the base;
candidate = your tree; one codegen unit, separate target dirs, offline,
serial):

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

## Director gate — A1 and A2 (2026-11-02)

**`gate-638a1` GREEN** (crates tree `581d68eabe0e`): scope, fmt, check,
ownership, census (RULED at 1066 then), oracle, standing, fingerprints,
certificates, inventories and the workspace suite (4,033 passed / 0 failed /
61 ignored) all passed with both LP03 shape items as written.

**`gate-638a2` GREEN** (crates tree `5e5f243b941c`):

- census RULED at **1067 (10/1022/35)** under R-638-1 and R-638-2: paired
  1064, retired 3, new 1, base-only 0, cand-only 2; definitions and roots
  stay 7707/7707. Every moved row is named in Amendment 5's ruling.
- oracle PASS after Amendment 6's constructor replay: 175 live contexts,
  177 with the two preserved clone-return fixtures, both builds.
- standing lp01 1/0 and typed-slot 10/0; fingerprints 68 contracts with 0
  changed; certificates pass in both builds with 0 promoted rows merged;
  inventories 19 passed / 0 failed with the frontier at 1067.
- **workspace PASS in both builds**, run by the director outside the
  sandbox: default 4,037 passed / 0 failed / 61 ignored (peak 13.5 GiB of
  14, oom_kill 0); `--no-default-features` 4,037 / 0 / 61 (peak 8.6 GB,
  oom_kill 0).
- A2 self-flagged one scheduling overlap (fmt and fingerprint generation in
  flight together, both green). It reached no measurement and no source
  decision; recorded here so it is not mistaken for a rerun artifact.
- The two helpers A2's adopted starts orphaned
  (`runtime_profile_outcome_for_option_result`,
  `runtime_profile_outcome_for_internal_result`) were deleted under
  Amendment 5. The only warnings left are the three pre-existing
  `register_test_*` ones.

**Amendment 7 (2026-11-02, Stage B STOP: LP04 pays on the default path).**
Stage B is right and the verdict stands: LP04 is not acceptable as it is.

Measured, and reproduced from the retained tables:

- `try_apply_down_values_from_symbol` gains **26 own instructions per
  call** in both named routes (412 -> 438 on `downvalue-pattern-hit`,
  247 -> 273 on `downvalue-exact-hit`); the adopted RHS start gives back 10
  and 4, and the outer route 5, leaving +11, +15 and +19 on the perf rows.
  `logical_definition_rule_start` has no callgrind row in either arm, so the
  26 is inlined code, not a call.
- 43 ordinary and 30 resume labels are positive in both pairs;
  `appendto-list-incremental-small` is +5.57 %; all 22 debug stack shapes grow
  32-240 bytes/level (the release shrink does not waive them).

The cause to attack first: every rule site now pays **two**
`lexical_profile_enabled()` reads — the legacy `definition_rule_timing_start`
and the new `logical_definition_rule_start` — and holds a second live
`Option<Call>` across the RHS evaluation. LP03's accepted shape already shows
the fix: one adopted token, one check, both records opened in one `#[cold]`
half. For the rule record that means one starter per site (the legacy timing
token and the logical `Call` in one value, one gate), and one completer pair
that consumes it; the disabled path must be one predictable branch and the
same or smaller live value than the legacy `Option<Instant>` it replaces.

Required, in this order:

1. **Calibrate before designing.** Build a mutant whose eight logical openers
   return `None` unconditionally and nothing else changes, and measure the
   two named routes against base in the same A/B. If the mutant is not
   base-equal, the opener is not the whole cost: attribute the rest first and
   say so. Keep the mutant as a standing row in the report.
2. **Then the merged shape**, with the observation unchanged: same row names,
   same finish order (inner `definition.rhs.*`, then `definition.rhs.total`,
   then the rule), the same `InputRequired` local close, the four loss-ledger
   bars, 4,375 receipts, the census 1067 and the constructor 175/177.
3. **Re-run Stage B exactly as before**, on the same arms (base = the export
   at `.temp/director/gate-state/worktree`, candidate = your tree), and add
   the LP04 commit `4d4851b3` as a second candidate arm so the repair's own
   delta is visible. The bar does not move: no ordinary or same-tier per-call
   increase, all 22 debug and release shapes with zero-residual fits and no
   ordinary growth.

Do not waive a positive row as noise, and do not chase timing while the
instruction counts on the named routes are still above base.

**Amendment 8 (2026-11-02, second repair round: the token is right, the
argument is not).** The repair did its job where it was aimed: both named
routes improve in both samples and the mutant returns the scanner to
412 / 247 own instructions. The three remaining reds are one real residue and
one measurement gap.

1. **The token size is not the stack growth.** I probed the shape on this
   toolchain: `size_of::<Option<Instant>>()` and
   `size_of::<Option<DefinitionRuleTimingToken>>()` are both 16 bytes. Put
   that equality into the code as a `const` assertion (a compile-time
   guarantee, not a comment) so no later edit can grow the slot silently.
2. **The debug growth is the by-value `Symbol`.** The starter takes
   `symbol: crate::runtime::Symbol` by value — 24 bytes spilled at each of the
   eight sites in debug, which matches the +32 to +80 bytes/level the probes
   see while release shrinks everywhere. Take `&Symbol` and clone only inside
   the cold half, then re-measure. If any shape still grows, name the frame
   that holds it (per shape, from the probe heights) instead of arguing from
   the token.
3. **The panel needs a noise control, not a waiver.** The three-arm columns
   already show the same label flipping sign across arms (`lazy-part-small`
   repair-base `[206.2, 75.5]` against LP04-base `[-110.1, 194.2]`), so the
   absolute "any positive is a defect" reading flags the measurement, not the
   code. Add a **base-vs-base control arm** — the export built twice at the
   same path with one codegen unit, two samples — and report per label
   whether the repair-base delta lies outside that control's spread. A label
   counts as a regression when it does, or when a deterministic own-Ir row
   reconciles a positive count x cost. The bar itself does not move; the
   control decides what is a delta at all.
4. **Re-run Stage B** on the same arms plus the control, and add the timing
   panel only after (2) is settled: the timing lane is secondary and must not
   be chased while instructions or stack are open.

Everything else stands from Amendment 7: the observation is unchanged (row
names, finish order, the local `InputRequired` close, the four ledger bars,
4,375 receipts, census 1067, constructor 175/177).

**Amendment 9 (2026-11-02, round 2 accepted where it aimed; the control was
not an instrument; the own-Ir rows are the residue).**

Accepted from round 2, re-read from the retained tables:

- **Stack is closed.** All 44 fits are zero-residual and no shape grows
  against base: control equals base in every row, debug repair is -32..-80
  bytes/level, release -160 or 0, and the two +32 release rows are 0 against
  LP04. The 15 static entry reservations that grew +16..+48 (largest:
  `try_apply_up_values` +48 and `try_apply_sub_values_from_symbol` +48) are
  supporting evidence, not a bar — the live chain is the bar and it does not
  grow. Round 3 re-checks the +48 on the recursive `try_apply_up_values` and
  names the frame if it survives.
- The 16-byte const assertion is in the code; copy, allocation, layout and
  lifecycle pass; both named routes improve in both samples
  (`downvalue-pattern-hit` -16.19/-16.07, `downvalue-exact-hit`
  -13.01/-13.39).

**R-638-3: the two-sample control interval is not a noise bound, and the 132
above-control labels are withdrawn as regressions.** This is an error in
Amendment 8 (3), my instrument, not a finding about the code.

- The null rates are exact under exchangeability:
  `P(max(repair pair) > max(control pair)) = 1/2`, and
  `P(both repair samples above the control max) = 1/6`. Observed: **132**
  against 156.5 expected, and **30** against 52.2 expected — both *below*
  chance. A 42 % flag rate on a panel whose null rate is 50 % reports the
  measurement.
- A two-draw control does not even bound the same-binary spread. On **183 of
  313** labels, some arm's two runs of the *same binary* differ by more than
  the control pair does. Median same-binary run spread on the candidate arm
  is 44.4 instructions/iteration; `lazy-nestlist-realize-small` differs by
  50.5 between the candidate's own two samples while its control pair differs
  by 1.3; `replaceall-simple` by 708x.
- Standing lesson, unchanged: a one-pair floor flags half the panel. The
  replacement instrument is item 4 below.

**The five deterministic own-Ir rows stand, and they are the STOP.**

| Label | Function | Base -> repair own/call | Extra per call |
| --- | --- | --- | --- |
| downvalue-guarded-hit | finish_suspended_down_value_application | 173 -> 174 | +1 |
| default-unguarded-hit | finish_suspended_down_value_application | 157 -> 158 | +1 |
| default-guarded-hit | finish_suspended_down_value_application | 157 -> 158 | +1 |
| default-unguarded-miss | try_apply_down_values_from_symbol'2 | 104 -> 106 | +2 |
| upvalue-guarded-hit | try_apply_up_values'2 | 119 -> 121 | +2 (x2 per iteration) |

This route is immune to run noise: base and control opcode counts are
identical at the same entry address, every executed PC has a fixed integer
multiplicity per call at N and 2N, and the reconciliation is count x cost.
All five labels improve in total in both samples; a net label gain does not
waive a positive per-function row. An off-by-default feature may not leave a
per-call cost in a function.

**The shape: the off-path token must be one word.** `Option<Call>` is 8
bytes and `Option<Instant>` is 16; the merged token is 16, and the residual
rows carry the two-word signature — exactly +2 mov per call in each
recursion context (one store, one reload) and a +5 mov / +1 movl / -6
reshuffle on the completion path. Box the whole record so the disabled start
returns a null pointer:

```rust
pub(in crate::runtime) struct DefinitionRuleTimingToken(Box<DefinitionRuleRecord>);

pub(in crate::runtime) struct DefinitionRuleRecord {
    start: Option<std::time::Instant>,
    call: Option<Call>,
    _storage: Option<Storage>,
}
```

with `size_of::<Option<DefinitionRuleTimingToken>>() ==
size_of::<Option<Call>>()` as the const assertion (8 bytes), replacing the
16-byte one. The slow half keeps every field; the legacy-only case is the
same box with `call: None`. Keep `&Symbol`.

**Diagnostic checkpoint before the campaign.** Rebuild base, control and
candidate, then run only `downvalue-guarded-hit`, `default-unguarded-hit`,
`default-unguarded-miss`, `default-guarded-hit` and `upvalue-guarded-hit`.
Require `finish_suspended_down_value_application` at 173/157,
`try_apply_down_values_from_symbol'2` at 104 and `try_apply_up_values'2` at
119 own instructions per call — base-equal — before the 313-label panel
runs. If a row survives the one-word token, test the 2x2 on those five
labels only — token (16 vs 8 bytes) x symbol argument (by-value vs borrow) —
and report the own-Ir result *and* the 44 stack fits for the cell; do not
run the campaign on a shape that has not cleared the five rows. Either way,
name the extra instruction from the address review.

**Stage B's panel instrument (replaces Amendment 8 (3)).** Five samples per
arm, interleaved round-robin across base, control and repair so drift cannot
masquerade as a delta; LP04 keeps its two samples as a reference column
only, not a decision input. A label is a regression iff
`min(repair deltas) > max(control deltas)`. The per-label null rate is
`1/C(10,5) = 0.4 %`, so at most about 1.3 false flags are expected across
313 labels, and a single flag escalates to more samples on that label
instead of stopping the stage. Report every sample, not an interval.

Timing stays last, after the five rows are at base and the panel is green.
Everything else stands: the observation is unchanged, census 1067,
constructors 175/177, 4,375 receipts, the four ledger bars, the 44 stack
fits green.

**Amendment 10 (2026-11-02, round 3 accepted where it aimed; the merged
*value* is the wrong abstraction, the merged *gate* is the right one).**

Accepted from round 3, re-read from the matrix and address proof:

- **The one-word token works where it was aimed.** The three completion rows
  are now *below* base: 172/156/156 against 173/157/157 (-1 per call), in
  both argument forms once the token is 8 bytes. Production keeps the
  8-byte borrowed cell.
- **Stack stays closed in production**: 528 correct probes, 44
  zero-residual fits, no growth against base. The release up frame is
  1008 -> 1024 (+16) and `evaluate_definition_application_rhs` is -112 per
  level, netting the measured -96. The two scratch by-value cells grow (21
  and 2 shapes) and were not promoted; the +48 entry reservation is a static
  bound, not the live chain.
- **The token shape does not explain the two recursion rows.** All four
  cells (token 16/8 x symbol value/borrow) keep `+2` per call in both
  recursion contexts. The exact added instructions are named: down-value
  reloads the spilled session pointer twice from `rsp+0x10`; up-value copies
  `rsi` to `r12` and reloads the saved pointer from `rsp+0x340` into `rbx`.
  This is a register-allocation consequence of the merged value, not a
  second gate read and not the symbol argument.

**R-638-4: keep one gate, keep base's two values.** Amendment 7's "one
token per site" was my instruction, and the measurements retire it: base's
own two-value shape allocates no spill in either recursion context, while
every merged-value cell does. What round 1 actually needed was one *gate
read* per site, not one value. Merge the gate and leave the values as LP03
had them:

```rust
let enabled = crate::runtime::profiling::lexical_profile_enabled();
let profile_rule_start = if enabled {
    crate::runtime::profiling::definition_rule_timing_start_under_gate()
} else {
    None
};
let logical_rule = if enabled {
    self.logical_definition_rule_start_under_gate("down_value_rhs_eval", &symbol, index)
} else {
    None
};
let result = self.evaluate_definition_application_rhs(option_scope, rhs);
record_profile_definition_rule_timing(
    "down_value_rhs_eval", &symbol, index, profile_rule_start, result.is_ok(),
);
complete_definition_rule(logical_rule, &result);
```

- Both `_under_gate` entries are `#[cold] #[inline(never)]` and *assume*
  the gate already passed; the legacy one calls the existing slow path, the
  logical one is `logical_definition_rule_start_slow` with `&Symbol` and the
  clone inside. Nothing but the gate is inlined at a site.
- The disabled path is one gate read (two relaxed loads and a branch), two
  `None` values — exactly LP03's live set — and no call. Delete
  `DefinitionRuleTimingToken`, `DefinitionRuleRecord` and the const
  assertion; the Box per rule goes away with them.
- The observation is unchanged: legacy start first, logical start second,
  the same completion order (legacy timing record, then the rule), the same
  row names, the four ledger bars, 4,375 receipts, census 1067,
  constructors 175/177.

**Checkpoint, then the panel.** Rebuild base, control and candidate and run
only the five labels. Require `finish_suspended_down_value_application` at
or below 173/157, `try_apply_down_values_from_symbol'2` at 104 and
`try_apply_up_values'2` at 119, and the 44 stack fits zero-residual against
base. If that holds, run Stage B with Amendment 9's instrument (five
samples per arm, round-robin, `min(repair) > max(control)`), then copy,
allocation, layout and lifecycle, then timing last. If it does not hold,
report the exact added instructions and stop; do not improvise a fifth
shape.

**Amendment 11 (2026-11-02, Q1 answered: no new export, no scope change).**

The scope STOP is right, and the remedy is not to widen Scope. The
`_under_gate` legacy entry is unnecessary: `definition_rule_timing_start()`
is already exported to the runtime and already checks the gate in its own
inline head, so calling it *under* the site's `enabled` flag reads the gate
once on the disabled path (the enabled path re-reads, and CSEs; it is not
the hot path). The logical under-gate opener is the existing slow body in
`logical/mod.rs`, made callable at `pub(in crate::runtime)` and keeping
`&Symbol` with the clone inside:

```rust
let enabled = crate::runtime::profiling::lexical_profile_enabled();
let profile_rule_start = if enabled {
    definition_rule_timing_start()          // already exported; re-checks on the enabled path only
} else {
    None
};
let logical_rule = if enabled {
    self.logical_definition_rule_start_under_gate(kind, &symbol, index)  // cold, &Symbol, clones inside
} else {
    None
};
let result = self.evaluate_definition_application_rhs(option_scope, rhs);
record_profile_definition_rule_timing(kind, &symbol, index, profile_rule_start, result.is_ok());
complete_definition_rule(logical_rule, &result);
```

Every file this needs is already in Scope; `crates/rum-runtime/src/runtime/
profiling/mod.rs` stays untouched. Everything else in Amendment 10 stands,
including the checkpoint and the ban on a fifth shape.

**Amendment 12 (2026-11-02, round 5 brackets the design space; the one-word
token is its minimum).**

Round 5's checkpoint failed three of five rows and the stack, and it
measured the exact cost of the shape I ordered:

- The three completion rows are **196/180/180 against base 173/157/157
  (+23/call)**, and 21 debug shapes grow +32..+96 bytes/level. The added
  instructions are named: eight 16-byte loads and eight 16-byte stores copy
  the **128-byte `Result` carrier** at `0x57dc24..0x57dc96` before the tag
  branch, plus seven net opcode changes. Two live values (the legacy
  `Option<Instant>` and the logical `Option<Call>`) force the carrier copy —
  the same class as LP02's large-result copy.
- Both recursion rows are **exactly base**: down 104, up 119. That is the
  other half of the bracket, and it is why Amendment 10 looked right.

The two shapes bracket the design space: **one value carried across the RHS
costs +2/call of register spill in the two recursion contexts; two values
cost +23/call of carrier copy plus 21 growing stack shapes.** The one-word
token is the minimum by an order of magnitude and the only shape that keeps
all 44 stack fits.

**R-638-5: keep the Amendment 9 candidate; the two recursion rows are a
named, accepted residual.** `try_apply_down_values_from_symbol'2` +2/call
and `try_apply_up_values'2` +2/call (x2 per iteration) are recorded with
their count x cost as the price of the merge. They are not waived as noise —
they are deterministic, PC-mapped, and they survive every measured cell —
and the only shape that removes them was built and measured at 23/call plus
21 growing stack shapes. Any *new* positive own-Ir row is still a STOP, and
the five affected labels face the panel's label test like every other label.

**Resume from the retained cell.** Restore exactly the Amendment 9 candidate
(`.temp/brief-638/amend9/sources/candidate`, the 8-byte borrowed shape),
verify it against `amend9/candidate-source-hashes.json`, and re-run the
five-label checkpoint to confirm 172/156/156 completion and 106/121
recursion before the campaign. Then Stage B: the 313-label panel with five
interleaved samples per arm and `min(repair) > max(control)`; copy and
allocation on 192 labels; layout, lifecycle and prologues; then timing last.

The observation is unchanged: legacy start first, logical start second, the
same completion order, the same row names, the four ledger bars, 4,375
receipts, census 1067, constructors 175/177.

**Amendment 13 (2026-11-02, round 6's instrument is sound; the residue is a
class, and two labels need full-N attribution).**

Round 6 ran the panel the way it should be run: 313 labels, five interleaved
samples per arm, four initial flags, five more samples each. Stack and
copy/allocation pass. Layout, prologues, lifecycle and timing were correctly
left unrun behind the instruction gate. Two things remain.

1. **The own-Ir residue is one class, not two rows.** The 192-label census
   found the same two reloads (`0x574142`, `0x574172`, `mov 0x10(%rsp),%rax`
   and `mov 0x10(%rsp),%rdx`, once per call at N and 2N) in two more
   contexts: the unprimed scanner `try_apply_down_values_from_symbol`
   (412 -> 414 on `downvalue-pattern-hit`) and a second primed path
   (396 -> 398). **R-638-6: the accepted residual covers every context that
   executes those two reloads**, at +2/call. It is the same instruction, the
   same mechanism and the same count; naming only two functions was too
   narrow. The justification is unchanged and is measured, not argued: the
   only shape that removes them costs +23/call and 21 growing stack shapes
   (Amendment 12). Any positive own-Ir row that is *not* these two reloads
   remains a STOP.
2. **The two persistent labels are real under the instrument and must be
   attributed.** `dot-three-no-input` separates by 6.15 instructions per
   iteration on a base slope of 248,506.6 (0.0025 %); `failure-tag-fallback`
   by 20.94 on a base slope of 9,427.6 (0.22 %, candidate deltas 27..41).
   Both are repeatable across ten samples with a control spread of about
   one instruction, so neither is noise. The capped census (N=80 and N=500)
   does not reproduce the panel and, as reported, does not explain them.

**Round 7 is a diagnostic, not a new shape.** Run base, control and
candidate under Callgrind at each label's exact panel N (`dot-three-no-input`
N=20000, `failure-tag-fallback` N=500), and reconcile the delta per
function and per PC. Either the separation is the named reloads times the
incoming calls (plus a reshuffle that nets zero), in which case the label is
the accepted class and the gate closes, or a new row carries it — name it,
with its PC and opcodes, and stop. Do not cap N, do not substitute the copy
census, and do not propose another shape.

If the gate closes: layout, prologues and lifecycle, then timing last.
