# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

**Archive note (2026-07-03):** the previous log (6386 lines, everything through
the long-tail/capability-sandbox/release-prep era) moved to
`docs/PROGRESS-archive-2026-07-03.md`. State at archive time: correctness
mainline complete (README 3126/0 reproduced exactly); host-embedding hardening
arc done (Tier-1 + Tier-2 core + rum-jupyter real interrupt); long-tail builtin
batches done and the support matrix accuracy-passed (brief-094); capability
sandbox landed (brief-095); public-release scrub method recorded and
`release-staging/` deleted pending re-creation (brief-096); release DEFERRED in
favor of code cleanliness. Active line: crate rebalancing, route 2a —
`docs/crate-rebalance-plan.md`.

## 2026-07-04 Hygiene-policy revision + coda batches DONE (briefs 120/121) — the whole line closes

The user asked for an evidence-based review of the size-hygiene policy
before continuing; the review's four amendments landed in
`docs/code-organization-guidelines.md` (4c88a44): quantified test-file
thresholds (15k/25k/45k) + identity-proof rule, a NEW Directory Fan-Out
section (~20 direct children soft ceiling), measure-implementation-after-
test-extraction, and assess-on-touch replacing the split-plan blanket-TODO;
plus an honest scoping of the incremental-compilation goal (crate is the
unit) and a "Splitting Is Organization" section codifying the brief-114
principles.

Then two coda batches applied it. Brief-120 (932c4d9): the four 30k+ files
split on real boundaries (iteration → spec/driver/table_fast_path with the
113 fast path housed intact; model → 9 type-domain modules; scoping/lexical
→ block/module/with; eval_core → symbol/call/stability) — 22 modules
≤17.4k, sentinel improved both A/B runs, the assess-and-keep option was
available but unneeded. Brief-121 (this commit): runtime/'s fan-out grouped
74 → exactly 48 via five fixed domain directories (definitions/, messages/,
session/, substitution/, kernel_data/ — the last collecting the rebalance
line's kernel-resident *_data machinery), transparent re-exports, zero body
changes, sentinel 55ns/iter. Workspace 1301/0 through everything.

## 2026-07-04 File-slimming line COMPLETE (brief-119) — the >35k debt tier is ZERO

The final batch split the two test files by BEHAVIOR FAMILY with a
test-name identity proof (stdlib 359=359, syntax 152=152, normalized diff
EMPTY): `incidental_runtime.rs` 67k → 12 behavior modules ≤12.2k (its
migration-artifact grab-bag decomposed into what is actually being tested);
`parser/tests.rs` 38k → 8 grammar-area modules ≤12.8k. Gate re-verified the
line's end state: the workspace-wide >35k audit is EMPTY (was 9 files two
days ago: profiling 138k, catalog 74k, incidental 67k, evaluation_cache
63k, definition_application 55k, vm_rhs_template 53k, arithmetic 42k, lexer
41k, parser/tests 38k); the 20-35k split-plan band stands at 26 files (the
next-granularity decision, not urgent). Workspace 1301/0 conserved through
all six batches; two hot splits carried independent gate A/Bs; zero
behavior change end to end. The splitting principles (by logic/function,
incremental-compilation-aware, future-refactoring-aligned, verbatim, split
map first) are codified in brief-114 for reuse.

## 2026-07-04 File-slimming batch 5 DONE (brief-118) — arithmetic.rs 42KB split by numeric domain

The hottest bodies split into 8 modules: `canonical.rs` (18.5k — Plus/Times/
combine_* kept COHESIVE per principle 1, the biggest-but-justified module),
exact (shared rational helpers), rounding, integer_ops, factorial, extremum,
parts, facade. Bodies verbatim; public surface (ArithmeticDataBuiltin,
eval_arithmetic_data_builtin, expr_as_exact_*, combine_*) unchanged; only two
sibling-shared helpers widened to pub(super). A/B: the brief-113 regression
SENTINEL table-iterator-plus at +1.7% (clean); do-small −5.7%; downvalue
cluster +2-5% inside the day's drift band. Gate re-verified: 1301/0
conserved, smoke 110, sentinel triple-probe 55-69ns/iter (matches the
post-113-fix baseline). Remaining debt: the two test files (batch 6, last).

## 2026-07-04 File-slimming batch 4 DONE (brief-117) — vm_rhs_template.rs 53KB + lexer.rs 41KB split

The opt-in VM split into 7 modules (model/compiler/executor/cache/safety/
tests + facade, ≤16.2k) with the disabled-gate check verbatim in cache.rs
and boundaries that keep the future feature-gate/move decision trivial. The
lexer split into 10 modules (core dispatch loop, cursor primitives, strings/
escapes, named_characters table, numbers, symbols, operators, trivia, tests
+ facade, ≤14k) with a clean DAG rooted at cursor/operators. Zero caller
changes (directory-module convention). The pre-existing source_template
dead-code warning was left for the VM-fate decision (the fix-in-passing
authorization was optional). Gate: 1301/0 conserved; lexer named-character
escape smoke (`\[Alpha]`) → 21. Remaining debt: arithmetic 42k (HOT, next),
the two test files.

## 2026-07-04 File-slimming batch 3 DONE (brief-116) — definition_application.rs 55KB split by value-application domain

The hottest file of the line split into 8 modules, all ≤14k: down_values /
up_values / sub_values (dispatch paths), fast_match (the hot candidate/
early-out layer, ISOLATED), default_values, rhs, replace_all (the rewrite
drivers the rewrite_data wrappers consume), mod (facade + shared). DAG:
dispatch paths → fast_match/rhs; replace_all → fast_match. Bodies verbatim;
visibility adjusted only where module boundaries force it. Codex's A/B first
run showed +3-13% across judged labels, receding on its rerun to ≤+5.6% on
two labels — because this is the hottest file, the GATE ran an independent
fresh A/B (HEAD worktree vs working tree): ALL judged labels within ±3.6%,
mostly NEGATIVE (downvalue-exact-hit −1.4%, replaceall −2.8%, matchq
−2.4~−3.5%) — drift confirmed, split neutral. Workspace 1301/0 conserved.
Remaining debt: vm_rhs_template 53k, arithmetic 42k, lexer 41k, 2 test files.

## 2026-07-04 File-slimming batch 2 DONE (brief-115) — evaluation_cache.rs 63KB split on the mechanism/instrumentation axis

The first HOT split held its red lines: `evaluation_cache/` now separates the
per-eval cache MECHANISM (`stability.rs`, 15.5k — impl Session decisions +
bloom) from the env-gated INSTRUMENTATION (`profile/{mod,source,report}.rs`,
the future move-to-profiling/ candidate) with a 1.6k facade + verbatim
tests.rs; all files ≤19.8k. Codex ran the A/B itself per the brief: machine
labels improved-or-flat, the suspicious downvalue cluster (+8%) fell to
+2-3% on the allowed rerun — no consistent regression; measurement-hygiene
note of the batch: it caught the microbench script reusing a STALE release
binary, discarded the polluted run, and rebuilt before measuring. Gate
re-verified 1301/0 conserved, pure-deletion diff (1681 lines), downvalue
smoke 25. Pre-existing dead-code warning noted in vm_rhs_template.rs
(source_template) — untouched, belongs to that file's own batch.

## 2026-07-04 File-slimming batch 1 DONE (brief-114) — profiling.rs 138KB + catalog.rs 74KB split

The slimming line opened (the debt deferred by the rebalance plan; user
principles codified in the brief: split by logic/function never mechanically,
optimize for incremental compilation — stable vs churning regions, sibling
DAG only — and for future refactoring — boundaries movable wholesale; pure
organization, split map first). Batch 1 took the two largest COLD files:

`runtime/profiling.rs` (138k chars, 151 fns) → `profiling/` with 17 modules,
all ≤14.8k: state/config/counters/scopes (stable declarations + env gates) →
records/timing/presence/fingerprints (instrument domains) → report_* +
summary + formatting (churny assembly). The split map survived one revision
(mod.rs initially 24k → stable state extracted). Disabled-path red line
asserted: same env-gated fast-outs, bodies verbatim. `stdlib/catalog.rs`
(74k) → `catalog/` with 14 per-family files (≤12.5k) + an 881-char facade;
install_core and the drift test compile UNCHANGED — and the bidirectional
attribute-drift test passing IS the strongest verbatim proof for the spec
data (gate also verified SET_OPERATION_ATTRIBUTES pre-existed and the 345
count). Zero >20k exceptions; zero caller changes (git status shows exactly
2 deletions + 2 new directories). Gate: fmt + workspace 1301/0; CLI smoke 61.
Remaining debt: evaluation_cache 63k + definition_application 55k (HOT, next,
with A/B), vm_rhs_template 53k, arithmetic 42k, lexer 41k, 2 test files.

## 2026-07-04 Stage-3 FINALE DONE (brief-113) — arithmetic moved; ALL 14 families' spec ownership in rum-stdlib

The line's endpoint: `builtin_catalog/numeric.rs` is GONE; the 27 hottest
handlers (Plus/Times/Power/…, + Identity) moved as wrapper-shape specs
(stdlib catalog 345); arithmetic bodies + `combine_*` canonicalization stay
kernel-side behind `ArithmeticDataBuiltin`. Kernel catalog residue is now
pure evaluator/host machinery: definition/rule machinery + Part family
(rules.rs), Function/HoldForm, scoping, namespace/context/loading,
debug/resource tools, 19 symbol-only string specs, known-symbol vocabulary.
A shared `#[cfg(test)] test_builtins.rs` now serves runtime mechanism tests.

THE GATE CAUGHT THE LINE'S ONE REAL PERFORMANCE REGRESSION here — the
mandatory finale A/B found `table-iterator-plus` at **14x** (63→907ns/iter)
while all 1300 tests stayed green (a pure fast-path-eligibility loss,
invisible to correctness). Root cause: the iterator fast path gates on
`session attributes == default_system_attribute_set(symbol)`; the static
side lost EVERY moved family's attribute data as catalogs left the kernel
(latent since batch 5, exposed only when Plus — the hottest symbol — hit the
check). Architectural conclusion recorded: symbol ATTRIBUTES
(Orderless/Flat/Hold*) are evaluator semantics and must stay kernel-known —
of the spec quintuple, handler and capability may move; static attribute
knowledge may not. Fix: kernel-side attribute vocabulary covering all 345
stdlib specs (112 non-empty) consulted as the fallback + a BIDIRECTIONAL
drift test (stdlib spec attrs == kernel vocabulary, no missing/no extra).
Fix verified: table-iterator-plus stable at −6~−10% vs base across two runs;
remaining table-wide +10-19% swings re-run-verified as machine drift
(receding + direction-flipping; the measurement environment ran ~15
compile+bench cycles today — noise floor ±10% acknowledged in the judgment).
Workspace 1301/0 (the drift test adds one); CLI smoke → 55.

## 2026-07-04 Stage-3 batch 11 DONE (brief-112) — rewrite_builtins: wrapper shape, engine untouched

Codex chose the brief-111 wrapper shape with clean evidence (the rewrite
bodies are matcher machinery: match/compile/normalize, rewrite-rule drivers,
traversal resolution, suspension drivers, trace hooks — a full move would
blow the promotion line): implementation renamed `runtime/rewrite_data*`,
7 specs moved (MatchQ, Cases, DeleteCases, Replace, ReplaceList, ReplaceAll,
ReplaceRepeated; stdlib catalog 318) behind thin wrappers via
`RewriteDataBuiltin` + one dispatch method (promotions: 2). Count/MemberQ/
FreeQ/Position/Level/Rule/NormalizeRules and the definition machinery stayed
(handlers not rewrite-module-owned). Tests: zero moved — mechanism tests use
test-local registrations per the amended rule. Workspace 1300/0; CLI → 7.

Gate ran the declared microbench A/B (wrapper covers hot ReplaceAll):
rewrite/matcher labels flat-to-faster (replaceall −1.6~−2.8% across two runs,
matchq −2.6~−4.2%, downvalue family ±2%). A +13% cluster appeared on the
SMALLEST sub-ms labels (loop-empty, do-iterator-null, literal-integer) —
re-run verdict: machine-state drift, not regression (second run halved it;
`do-small` which actually executes Do bodies was NEGATIVE; literal-integer is
untouched by both batches). Two wrapper batches, two neutral A/Bs — the
wrapper shape is performance-safe. Remaining: arithmetic (the finale).

## 2026-07-04 Stage-3 batch 10 DONE (brief-111) — control_flow: spec ownership moved, implementation stays

The highest-coupling family got a DIFFERENT solution shape, gate-reviewed in
depth and accepted: reference analysis showed nearly the whole implementation
is evaluator machinery (non-local-exit signals consumed by eval/VM paths,
loop iterators on `BlockSymbolState`, abort/cleanup on the interrupt state,
failure/confirmation helpers used by eval_core/eval_dispatch) — so the
implementation stayed kernel-side (renamed `runtime/control_data/`), while
the 35 SPECS moved to the stdlib catalog with thin wrapper handlers
delegating through one promoted facade
(`ControlDataBuiltin` enum + `Session::eval_control_data_builtin`). Semantics
of the split: registration ownership (no stdlib install → no If/Do) is
stdlib's; the machinery is honestly kernel. Promotions: 2 (vs ≤12 line).

Because the wrapper adds a per-call indirection on HOT handlers
(Do/CompoundExpression/If), the gate ran a full microbench A/B (HEAD worktree
vs working tree): control-relevant labels flat-to-faster (do-small −4.2%,
if-true −1.7%, do-count-null −6.8%), the few +5% entries are sub-ms jitter
with contradicting same-mechanism neighbors — NO consistent regression;
neutral confirmed. Also accepted with a SECOND discipline note: Codex kept
~12 mechanism-test files runtime-side with TEST-LOCAL control registrations
instead of migrating them per the Option-1 letter — the right instinct (it
matches the batch-5 exile-list concern) but a rule deviation it should have
STOPPED to report; the rule is now amended going forward: mechanism tests MAY
stay runtime-side with test-local registrations, listed per batch. Behavior
adaptation this batch: `$AssertFunction`/`PropagateAborts` added to the
known-symbol vocabulary (string option forms). Workspace 1300/0 conserved
(runtime 430, stdlib 358); stdlib catalog 311; CLI smoke → 22. Remaining:
rewrite_builtins, arithmetic.

## 2026-07-04 Stage-3 batch 9 DONE (brief-110) — functional_ops moved; apply machinery stays

The cleanest of the big moves: 42 specs out (Map/Apply/Fold/Nest/Select/
Array/Outer/Composition/…), and the runtime functional catalog is now
EXACTLY the evaluator-semantics residue — `Function` and `HoldForm`. The
callable-application machinery the dispatcher consults
(parse_composition/apply_composition_to_args, operator-wrapper apply,
comparison/Key/NearestTo wrappers) stayed kernel-side as
`runtime/functional_data.rs`; the pure-function apply machinery was never
touched. Promotions 7/8 (line held). Test migration surprisingly small — 18
fns (16 whole-file across 6 files + 2 partials) — because earlier batches had
already pulled the functional test hotspots. Workspace 1300/0 conserved
(runtime 510, stdlib 278); stdlib catalog 276; CLI smoke → 12; gate
cross-boundary smoke (kernel Function × stdlib Map/Total) → 14. Remaining:
control_flow, rewrite_builtins, arithmetic.

## 2026-07-04 Stage-3 batch 8 DONE (brief-109) — structure_ops moved; Part stays kernel

The reference-analysis split produced a bigger stay than any batch yet, and
rightly so: `Part`/`Extract`/`ReplacePart`/`ReplaceAt` — specs (in
`builtin_catalog/rules.rs`) AND implementation — stayed kernel-side as
`runtime/structure_data/`, because Part-lvalue assignment is definitions-code
machinery; their runtime tests stayed too (`tests/structure_data/`). The other
38 specs (Length/Flatten/Sort/Take/…) + implementation moved. Runtime catalog
279, stdlib 234; workspace 1300/0 conserved (runtime 529, stdlib 259); CLI
smoke → 7, and the gate added a cross-boundary smoke (kernel-resident
ReplacePart/Extract composed with stdlib-resident ops) → 104. Promotions 7/8 —
the resized stop line held this time. Test-local registration list grew:
scoping_gensym_gc now registers a minimal `First` (same pattern as the VM
SameQ). Next: functional_ops — Codex flags `targeted_map.rs` as depending on
kernel-side exact-path helpers; reference analysis before handlers move.

## 2026-07-04 Stage-3 batch 7 DONE (brief-108) — associations moved with the data/builtin split; 9 families out

The designed split held: the association DATA LAYER (entry forms,
parse/render, key lookup/apply, set-lvalue — everything referenced by
kernel-side code or not-yet-moved families) stayed kernel-side as
`runtime/association_data.rs`; the 33 builtin specs + family helpers moved
(`MissingQ` stayed, its call is kernel-adjacent). `eval_dispatch` untouched
(the split-correctness stop-loss never fired). Runtime catalog 317, stdlib
196; workspace 1300/0 conserved (runtime 577, stdlib 211); CLI smoke → 21.
Biggest migration so far, as predicted from the 28-file incidental footprint:
22 dedicated tests + ~40 measured moves (9 whole-file + 12 partial sources).

Two gate notes. (1) Promotions came to 6 narrow facades — ONE OVER the
brief's ≤5 stop line; Codex should have stopped and reported rather than
finishing (outcome harmless, all six justified, but the discipline slip is
noted; future briefs size the line to the family's audited kernel-import
count instead of a flat 5). (2) One behavior-equivalence adaptation was
required and gate-verified: the new `protected_or_known_definition_target_symbol`
facade adds a REGISTERED-spec branch alongside the static vocabulary — moved
builtins (no longer in the static catalog) must still count as known/protected
for AssociateTo; consistent with the brief-102 resolution-probe conclusion.

## 2026-07-04 Stage-3 batch 6 DONE (brief-107) — predicates moved; 8 families out

50 predicate specs moved (runtime catalog 350, stdlib 163); the family got its
free-fn conversion in transit; `builtin_catalog/predicates.rs` deleted whole
(`MissingQ` stayed — its spec lives in the associations catalog). Migration
measured 33 test fns (vs logic's 31 — the mild trend holds). Promotions 4
(≤5 line): the audit-predicted `invalid_call_with_arg_form` +
`collect_level_parts_owned` + 2 `CasesTraversalConfig` constructors.
Workspace 1300/0 conserved (runtime 644, stdlib 144); CLI smoke → 2.

Two gate-reviewed edge calls, both accepted with lessons: (1)
`associations/basic/predicates.rs` renamed to `key_predicates.rs` ONLY because
the brief's grep gate used a bare word — grep gates must be path-precise
(second wide-grep incident after brief-100's); (2) the VM template tests lost
`SameQ` and now register a TEST-LOCAL minimal SameQ (structural equality) via
the open registry — probes unchanged, and crate-internal mechanism tests
genuinely cannot migrate (they touch runtime internals). EXPECT this pattern
to recur when control_flow moves (VM tests likely use If/Do too); the
test-local registration list joins the exile list as future-cleanup tracking.

## 2026-07-04 Stage-3 batch 5 DONE (brief-106) — logic moved; the comparisons left the kernel

The user chose Option 1 for the load-bearing tier (tests migrate WITH their
families; no dev-dep cycle; runtime test tree trends mechanism-only), and the
first mass migration turned out MILD: measured reality was 31 test fns
(22 whole-file + 9 partial), not the feared hundreds — runtime tests lean on
comparisons far less than assumed. Logic itself was the lowest-increment move
(free-fn conversion + kernel-method pub-ification all done in 099/101):
21 specs moved, runtime catalog 400, stdlib 113; the leftover catalog slice
renamed honestly to `builtin_catalog/predicates.rs` (50 specs); obsolete
nand/nor adapter shims deleted. Workspace 1300/0 conserved (runtime 679,
stdlib 111); CLI smoke `If[Xor[True, False] && 1 <= 1, 42, 0]` → 42.

Promotions: 3 (within the ≤5 line) — the gensym-GC introspection counters
(`collect_and_sweep_dead_gensyms` visibility flip + 2 read-only count
accessors), needed because the gensym-GC MECHANISM test uses logic builtins in
its probes and migrated per the Option-1 rule. Noted for a future cleanup
line: mechanism tests exiled to the stdlib only because their probe
expressions use stdlib builtins could be probe-rewritten and repatriated
later; the rule "don't rewrite probes to dodge migration" was correct for
keeping this batch honest, but the exile list is worth tracking (currently:
scoping_gensym_gc partial, debug_tools_resources partial).

## 2026-07-04 Stage-3 batch 4 DONE (brief-105) — strings moved (largest batch); 6 families out

The biggest single family so far: `strings` (facade + 21 files ≈6.5k lines, 61
handler-backed specs) now lives in rum-stdlib (which gained the `rum-syntax`
dep for ToExpression/SyntaxQ). Runtime catalog 482 → 421 (19 symbol-only
string specs legitimately stay); stdlib catalog 92; workspace 1300/0; CLI
smoke `StringLength[StringJoin[{"ab","cd"}]] + ToExpression["3*4"]` → 16.
Tests: the whole `tests/strings/` directory + 19 measured-failure fns
(→ `tests/strings/incidental_runtime.rs`) + the FileTemplateApply capability
assertion moved with the handler.

The one structural judgment call, gate-audited in depth: ToString/TextString's
expression RENDERER is shared kernel machinery (message_format,
builtins_input, scoping, control_flow all render through it) — it stayed in
the kernel as `runtime/text_rendering.rs`. Verified: byte-identical to the old
`strings/text/render.rs` modulo visibility/docs, plus `string_form_value`
folded in from template.rs (its only caller is the renderer itself). No
formatting code was copied or forked. Promotions within stop-loss: 7
kernel_api exports (incl. `Capability`, `ToStringFormat`, `render_*`) + 7 pub
Session methods (incl. the suspension API `suspend_with_frame`,
`drive_evaluated_value`/`drive_internal_value`,
`enforce_active_memory_limit_for_projected_bytes`) — 10 of 14 were
audit-predicted. Stop-loss lines (>12 new promotions / >40 failing fns /
pattern-matcher entanglement) NOT hit: measured 20 failing fns, no
entanglement (the string-pattern compiler moved cleanly with the family).

## 2026-07-04 Stage-3 batch 3 DONE (brief-104) — calculus + polynomials moved; 5 families out

First multi-file directory family through the pipeline: `polynomials/`
(mod/candidates/core/core/factors/factorization/rational_function) moved with
its layout intact; `calculus` (the `D` handler) along. Runtime catalog
488 → 482, stdlib 31 (11+5+9+1+5); runtime greps clean; CLI smoke
`--eval "Coefficient[Expand[(x+1)^4], x, 2] + D[x^3, x] /. x -> 2"` → 18;
workspace 1299/0 (runtime 753 + stdlib 34; 4 calculus + 13 polynomials test
fns migrated). One family-boundary discovery handled correctly: the
`PolynomialQ`/`PolynomialExpressionQ`/`RationalExpressionQ` PREDICATE handlers
physically lived inside `polynomials/mod.rs` — they moved VERBATIM (md5-equal
bodies, gate-checked) into `runtime/predicates.rs` so the predicates family
stays runtime-owned; their catalog entries never left. Promotions:
`HoldAttribute` + `Session::invalid_call_with_message` pub;
`parse_indeterminate_expr` facade retargeted. Remaining Stage-3 families: 8
(predicates, logic, arithmetic, strings, structure_ops, functional_ops,
associations, control_flow, rewrite_builtins — the load-bearing tier; test
strategy per family by measured failing-fn count).

## 2026-07-04 Stage-3 batch 2 DONE (brief-103) — integer_digits + number_theory moved; 3 families out

First BATCH move (two families in one brief) through the proven pipeline:
`rum-stdlib` now owns `integer_digits` (5 handlers) and `number_theory` (9
handlers, incl. the Prime/PrimePi small tables and Pollard-rho path); runtime
catalog 502 → 488, stdlib catalog 25 (11+5+9); runtime greps clean; CLI smoke
`--eval "NextPrime[100] + FromDigits[{1,2,3}] + EulerPhi[10]"` → 228; workspace
1299/0. Promotions: `Session::{invalid_call_with_arg_range,
enforce_generated_element_limit_for_count, thread_call_args, push_message}`
now `pub` (+ `evaluate` documented); `kernel_api` re-exports
`num_integer::{Integer,Roots}` + `num_traits::{One,Signed,Zero}` additions.

Two process notes. (1) The NEW test-migration rule proved out: `number_theory`
tests moved as a whole file (8 fns), while `integer_digits` tests were embedded
in `tests/arithmetic.rs` — the rule "run the runtime suite after the move; every
MEASURED failing function moves whole to the stdlib" migrated exactly 3
functions, no guesswork. (2) The first dispatch of this brief was killed
mid-move by my own foreground-chained dispatch command (Bash tool 2-min
timeout); the half-done tree was rolled back cleanly and re-dispatched. That
failure mode is now MECHANICALLY blocked by a PreToolUse hook
(`.claude/hooks/guard-dispatch.py`): any `dispatch-brief`/`cdx exec` Bash call
that is chained, bare-`&` detached, or not run_in_background is denied by the
harness before execution (DRY_RUN=1 exempt) — verified live.

## 2026-07-04 Stage 3 OPENED and the MOVE pipeline is PROVEN (brief-102) — combinatorics lives in rum-stdlib

The first family physically left the monolith. `rum-stdlib` now owns the
combinatorics implementation (`src/combinatorics.rs`, 11 handlers converted to
free fns while moving, bodies verbatim), its specs (`src/catalog.rs`, 11
entries byte-identical to the deleted runtime ones modulo handler path), and
its 5 behavioral tests; `install_core` = runtime catalog (now 502 entries)
then stdlib specs. rum-runtime greps CLEAN for combinatorics. Pilot family was
combinatorics (not logic) because it has zero incidental test usage — logic
owns the comparison operators the kernel's own tests use everywhere.

Step-0 resolution probe (the one real unknown) PASSED: a builtin registered
under a name ABSENT from the static catalog resolves and dispatches through
`eval_text` — no vocabulary fix needed; the probe stays as a regression test
(`registered_builtin_resolution.rs`). Kernel promotions were restrained and
documented: `Session::{invalid_call_with_exact_arg_count,
combine_additive_args, combine_multiplicative_args}` now `pub`;
`kernel_api` re-exports `BuiltinSpec` + `num_traits::ToPrimitive` (spares the
stdlib a num-traits dep). rum-stdlib gained the `rum-core` dep.

Gate: fmt + workspace 1299/0 (runtime −5 moved tests +1 probe, stdlib +5);
CLI smoke `--eval "Fibonacci[10] + CatalanNumber[4]"` → 69 (the brief's
literal `eval` subcommand syntax was wrong — rum-cli takes `--eval`; noted for
future briefs). No microbench (registration source only). Remaining Stage-3
work: the other 12 families through the same pipeline; load-bearing ones
(logic/predicates/arithmetic/…) additionally need the test-migration /
dev-dep-cycle decision recorded in the memory notes.

## 2026-07-03 Stage-2 step 3 DONE (brief-101) — logic handlers are free functions

The pilot family's handlers no longer live in `impl Session`: 16 handlers
(truth.rs 8 + comparison.rs 8, incl. the private `eval_comparison_subset` and
the truth continuation helpers) are free functions
`fn eval_x(session: &mut Session, …)` with bodies VERBATIM (receiver-only
change); the facade re-exports them per-item; `builtin_catalog/logic.rs` and
the nand/nor adapters reference `crate::runtime::logic::eval_*` explicitly.
Scope note that shaped this brief: catalog file grouping ≠ implementation
module grouping (catalog/logic.rs references 69 handlers; the logic MODULE
owns 16 — the rest are predicates/structure_ops and stay untouched until their
family's brief). Gate: fmt + workspace 1298/0; zero `impl Session` under
logic*; no microbench (static call → static call, dispatch fn-pointer shape
unchanged). The logic family is now move-ready: brief-102 = the physical move
pilot (files → the stdlib crate, spec ownership, kernel Session-method
pub-ification, test migration).

## 2026-07-03 Stage-2 step 2 DONE (brief-100) — registration is the ONLY builtin dispatch path

Discovery that motivated it: dispatch resolution silently fell back to a static
`OnceLock` map over `core_builtin_specs()`, so builtins never needed
registration and `rum-stdlib::install_core` was decorative — families cannot
leave the crate while the kernel statically consults its own catalog. Now:
`resolution.rs` reads ONLY the session's registered map (fallback deleted);
`Session::default()`/`new()` auto-install `core_builtin_specs()` (the
transitional bridge — shrinks as families move out); `with_config()` builds a
truly RAW session (hosts/stdlib must install — they already do);
`register_builtin` applies the `with_fallback_dispatch` merge against the
previously REGISTERED spec, preserving user-override semantics; `Names`
enumerates registered specs. The static `default_builtin_dispatch_spec*`
helpers legitimately REMAIN as vocabulary (known-system-symbol predicates in
`expr_runtime_helpers.rs` — a dispatch-vs-vocabulary distinction the original
brief's grep gate missed; Codex stop-lossed on it and the gate was amended).
New test: fresh session evaluates a builtin + DumpSave/Get restore keeps the
registered set. Gate: fmt + workspace 1298/0; resolution grep = 0; microbench
A/B flat-to-better (−8%…+1.6%, no consistent regression; mechanism touches
only the cache-miss resolution path).

## 2026-07-03 Stage-2 step 1 DONE (brief-099) — kernel_api v1 exists, logic family de-globbed

`runtime/kernel_api.rs` is now the single public door to the kernel's helper
surface: 40 audited fan-out ≥2 items re-exported in 14 grouped `pub use` lines
(`pub mod kernel_api` from `runtime/mod.rs`; defining `mod`s stay private, so
the promoted `pub` items are unreachable except via kernel_api). Pilot: the
`logic` family (`logic.rs` + `logic/{comparison,shared,truth}.rs`) carries NO
`use super::*` — only explicit `kernel_api`/`rum_core` imports. The pilot
CONFIRMED the audit model: demanded exactly the predicted 9 items (5 kernel_api
+ 4 rum_core), zero unplanned promotions.

Learned (now standing for briefs 100+): promoted items sit in private
submodules behind family facades; Rust cannot re-export a restricted re-export
(E0364/E0365), so facades need per-item `pub use self::<submod>::item;` lines
(never widen a glob re-export) — facade files belong in MAY-modify from the
start. Codex stop-lossed correctly on this and was resumed with a scoped
authorization. Gate: fmt + `cargo test --workspace` 1297/0; visibility/imports
only (rustfmt reflow aside, no body/signature changes; no microbench — no
codegen delta); the `SymbolExprMap`-vs-private-hasher warning closed at gate
time by making `RuntimeCacheHasher` `pub` inside its private mod (one word +
comment). Diff: 21 files, ~+56/−51.

## 2026-07-03 Stage-2 audit COMPLETE (brief-098) — the real kernel API is tiny

Method: per-family compile-error audit (comment out `use super::*` in each of the
14 builtin family modules, one at a time; collect unresolved names from
`cargo check`; restore) + a grep axis for kernel `Session` methods called from
family code. Full artifact: `docs/dev/kernel-api-audit-2026-07-03.md` (from
`/tmp/cdx_brief_098_report.md`); raw logs `/tmp/cdx_brief_098_raw_<family>.log`.
Diagnostic only — tree clean, final `cargo check` green. (Operational note: the
first run stopped at 6/14 families because Claude's concurrent PROGRESS-archive
commit tripped the whole-tree clean-check; resumed with the check scoped to
`crates/`. Lesson recorded: no repo mutation while a brief is in flight.)

Findings (free-item axis): of the runtime-internal namespace, only **258 items
are used by any family at all**; **24 have fan-out ≥3 families** (the prelude
core: `Session`/`EvalError` types, `system_expr`, `is_system_symbol_name`, the
`expr_as_*` coercion helpers, `compare_expr_canonical`, a few model/session
types like `ThreadingDispatch`/`ThreadArgSpec`/`CasesLevelSpec`), 16 have
fan-out 2, **218 are single-family** (move WITH their family, don't promote),
and ~600 internal symbols are used by no family (stay kernel-private). Known
family→family couplings that constrain move order: `arithmetic` numeric
helpers (`expr_as_exact_integer` 7 families, `expr_as_exact_rational` 5),
`associations/data` helpers (`association_entries_from_expr`,
`render_association` — 6 families each), structure path helpers.

Findings (Session-method axis, the future pub method surface, top of ~15-25):
`evaluate` (148 calls), the `invalid_call_with_*` argument-error helpers
(131+113+112+67), `suspend_with_frame` (100) + `drive_internal_value` (41) +
`suspend_with_result_frame` (17), `push_message` (37), the `enforce_*` resource
limits, `match_expr_owned`, `merged_option_exprs_for_system_symbol`.

Cheapest first move: **`logic`** — 9 free items, no same-family glob fallout, no
other-family coupling. Count reconciliation note: the earlier "528
`pub(in crate::runtime)` symbols" was a grep line count; the audit's item-name
scan counts 651 unique names — same order of magnitude, the design conclusion
(promote ~40, not 500+) is insensitive to the method.

## 2026-07-03 Crate-rebalance line OPENED (route 2a) — Stage 1 DONE: builtin dispatch is open

Line context: `rum-runtime` had grown into a 116,293-line / 391-file monolith
(evaluator + every builtin family) while `rum-stdlib` was a 21-line installer
facade; builtins were unmovable because dispatch was a CLOSED ~470-variant
`BuiltinHandler` enum + a three-file chained `match` + an exhaustive per-variant
capability `match`, and the families compile against a 528-symbol
`pub(in crate::runtime)` shared namespace via `use super::*` (377 files).
Canonical plan: `docs/crate-rebalance-plan.md` (d49e101) — Stage 1 open the
dispatch, Stage 2 define the kernel API, Stage 3 move the families; file-slimming
deliberately AFTER the split.

**Stage 1 (brief-097, commit 45d2e8a): delete the closed enum.** `BuiltinSpec`
now carries the handler as a plain fn pointer
(`pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>`)
and the capability as data (`capability: Capability`, default `Pure`; 8 FileRead
+ 3 FileWrite annotated in the catalog). Deleted: the `BuiltinHandler` enum, the
`eval_dispatch/{surface,symbolic,control_functional}.rs` match chain (~530
lines), and the `impl BuiltinHandler::capability` match (~500 lines); net −1467
lines. The 513 catalog entries reference `Session::eval_*` directly (fn-item →
fn-pointer coercion in const arrays); ~25 non-standard arms (args-only
signatures + inline bodies) live VERBATIM as adapters in `builtin_adapters.rs`
— `Evaluate` keeps its `suspend_with_frame` suspension shape. `CallDispatchPlan`
carries the resolved `BuiltinSpec` (Copy, resolve-once-at-plan-build preserved);
capability inheritance follows handler inheritance in `with_fallback_dispatch`
(a re-registered `Get` cannot shed its `FileRead` requirement). The
fn-pointer-comparison lint is allowed locally at the type (all spec copies
originate from one catalog entry, so pointer equality is stable).

Gate evidence: `cargo fmt --check` + `cargo test --workspace` green (1297/0);
`grep -rn "BuiltinHandler" crates/` = 0 hits; microbench A/B run TWICE
independently (Codex once, Claude reproduced via a HEAD worktree). KEY
measurement lesson: the two A/Bs disagreed label-by-label IN BOTH DIRECTIONS
(labels reported −21% in one run showed +9–11% in the other, and vice versa) —
sub-ms dispatch-tier deltas are noise-dominated; judge only cross-run
consistent-direction movement, of which there was none. Verdict: match-jump →
indirect call is measurement-neutral; the extraction precondition (builtin =
data registered into the session) now holds.

**Stage 2 audit dispatched (brief-098, e251618):** diagnostic, no source change
— per-family compile-error-driven audit (comment out `use super::*`, collect
unresolved names, restore) over 14 family modules + a grep axis for kernel
`Session` methods called from family code. Output: the promotion-candidate list
for the explicit kernel API (≥3-family items → prelude; single-family items →
move with the family; unused internals → stay private) + the family→family
couplings that constrain move order.

## 2026-07-04 — Release transformation line STARTED (plan committed)

Decision (user, this session): drop the fresh-history release-staging approach;
transform `main` itself into release shape, keep full history. New testing policy:
rum tests only its own documented design + rum-vs-rum perf A/B; difftest and the
mma-hol snapshot pipeline are removed (archived privately at
../Archive/rum-history, MANIFEST there; WLBuiltin/ already moved). Support matrix
dropped in favor of a hand-written language guide + a builtin reference generated
from the stdlib catalog + rustdoc. Endpoint: clean tree + bundle cold backup (no
remote). Plan: docs/dev/release-transformation-plan.md (briefs 124-130 + M0-M2).

## 2026-07-04 — M1: narrative + policy rewrite (release transformation)

DISCLAIMER.md rewritten (user-reviewed): independent project, no compatibility
claims, factual history of the early black-box method and its July 2026
removal, standing policy. README repositioned (embeddable sandbox-first
term-rewriting language; external-comparison framing removed). AGENTS.md
rewritten around the new Testing Policy hard lines (own-documented-behavior
tests + rum-vs-rum perf A/B only; no oracle harnesses/compat matrices — 
permanent). testing-strategy.md rewritten (4 layers, coverage axes kept,
documentation-as-contract, llvm-cov guidance). TODO.md rewritten (release line
active items + post-cut queue: toolchain refresh to current stable, coverage
rebuild, scoped-fs profile, pyo3-on-rum-host, showcase, vm_rhs_template).
CLAUDE.md updated to the new world. Briefs 124-126 all gated green earlier
today (commits 5eddf1b, 131614a, 3814ef0).

## 2026-07-05 — M1.5: ARCHITECTURE + v0.1 boundary rewritten; toolchain settled

ARCHITECTURE.md: independent positioning, rum-stdlib section updated to the
post-rebalance reality, rum-difftest section removed, Compatibility Target →
Behavior Contract (documentation is the contract), Performance Kernel
Direction → Performance Discipline (rum-vs-rum A/B), Conformance Strategy
removed, core-language boundary now defined by rum's own guide/reference.
docs/v0.1-acceptance-boundary.md rewritten (acceptance = documented slice +
hardened rum-host + CLI/REPL + jupyter preview; external-corpus lanes gone).
Toolchain/MSRV SETTLED per user: stay on Rust 1.85 (Debian stable default
rustc + edition-2024 floor); recorded in TODO/README. brief-127 gated green
(e0448c9): crates wording scan, zero behavior change, suite totals identical.

## 2026-07-05 — briefs 128 + 129 gated green (reference + guide)

brief-128 (8503321): generated builtin reference — 513 entries from the
installed registry (attributes, capability class, summary, verified limits
notes) + drift test (negative check demonstrated); determinism reproduced at
the gate (double run, empty diff); minimal doc-hidden read-only iteration
APIs in rum-runtime. brief-129 (64b7c08): nine-chapter language guide, all
examples current-build transcripts, nine spot-reproduced at the gate; README
links guide + reference. Guide writing surfaced two real pre-release bugs
(now in TODO §Pre-Release Fixes): unbounded user recursion natively
overflows the stack instead of tripping recursion_limit; ReplaceRepeated
limit surfaces as a CLI hard error instead of TerminatedEvaluation.

## 2026-07-05 — brief-130 gated green; fs-expansion direction queued

brief-130 (2089d35): rum-host fully rustdoc'd under warn(missing_docs) (zero
warnings, doctest green; one grouped re-export split into per-item pub use —
semantically identical), crate-level overviews for the library crates,
workspace metadata (descriptions, license/keywords/categories inheritance, no
vendor terms, repository field deferred to publication), ISSUE file archived.
Workspace 1131/0. TODO gains the post-cut fs-surface expansion item (scoped
roots first). Next: brief-131 fixes the two guide-surfaced bugs before M2.

## 2026-07-06 — brief-131 + M2: the release transformation line is COMPLETE

brief-131 (cdeaa59): Phase A measured ~20 KiB native stack per WL recursion
level (overflow at ~depth 412 of limit 1024) → native_stack_limit guard
(default 6 MiB consumed) at the amortized poll point, recoverable
TerminatedEvaluation["StackLimit"]; ReplaceRepeated overflow now
TerminatedEvaluation["IterationLimit"]. Zero new unsafe; workspace 1136/0;
A/B no hot-label regression (median 1.018, reproduced at the gate). M2:
TEMPLATE.md rewritten to the new policy; briefs 124-131 + the plan doc
archived; final sweep = whitelist-exact (DISCLAIMER nominative, three
one-sentence family mentions, jupyter ecosystem identifiers); TODO/CLAUDE
updated; bundle cold backup follows this commit.

## 2026-07-06 — FS purity line STARTED

Decision (user): strip the six general file builtins (FileNames, FileExistsQ,
FileDate, FileByteCount, Import, FileTemplateApply) out of the repo entirely
(archived for a future downstream rum-fs project, post-launch); keep the
module system (Get/Needs/FindFile) and persistence (Save/DumpSave) with their
I/O injected via a host provider; kernel crates become std::fs-free by
construction with a hygiene test. Plan: docs/dev/fs-purity-plan.md
(briefs 132-133 + M0/M1).

## 2026-07-06 — FS purity line COMPLETE (briefs 132-134)

brief-132 (a46836e): the six general fs builtins stripped and archived as the
future downstream rum-fs seed; pure path-string helpers stay; reference 507.
brief-133 (06dbbb6): FileIo provider seam — Get/Needs/FindFile/Save/DumpSave
do I/O through a host-injected trait; StdFileIo lives in rum-host (absent
under sandboxed()); General::noio contract for permissive-policy-no-provider;
hygiene test forbids std::fs in kernel crates outside /tests/ paths.
brief-134 (c92b84f): Capability::Clock — audit rule "Pure iff observable
result depends only on session state + arguments" moved 8 clock readers out
of Pure (incl. Pause/EvaluationData/TestReport the audit caught);
MemoryConstrained stays Pure with evidence; BlockRandom removed as an inert
shell; sandboxed() is now deterministic by construction; docs adopt the two
claims (no ambient authority; deterministic by default). Reference 506;
workspace 1141/0. Process docs archived; kernel hard lines now three: zero
unsafe, zero std::fs (non-test), Pure=deterministic.

## 2026-07-06 — brief-135: scripting-ergonomics probe (diagnostic, no source change)

Six embedded-scripting tasks all completed in current rum (log summary, text
transform, quoted-CSV munging via a hand-written character parser, config
validation, template report, word frequency). Friction: no raw multiline
string literals; CSV needs a hand parser; reached-for-but-missing: Mean,
Tally, MaxBy/TakeLargestBy, ReverseSort, StringPadRight, StringLines.
String-pattern matrix: the engine supports ~~, alternatives, blanks, .., 
character classes, Except, Shortest/Longest, named binding across all six
string functions — almost none of it cataloged (dark surface); inert:
PunctuationCharacter, StartOfLine, EndOfLine, WordBoundary; StringSplit lacks
IgnoreCase; four bug-like edges recorded. Batches 136-140 planned in TODO.

## 2026-07-06 — brief-136 gated (12 builtins); codec/hash/rum-os planned

brief-136 (ba4ad60): Tally, MaxBy, MinBy, TakeLargestBy, TakeSmallestBy,
ReverseSort, ReverseSortBy, GatherBy, Riffle, exact Mean, StringPadRight,
StringLines. Codex stop-lossed correctly on the kernel attribute-vocabulary
registration (fenced out by the brief); completed at the gate, plus
attribute alignment to the family norm (ordinary, not Protected — the brief
was wrong). Reference 518; workspace 1151/0. TODO gains: codec family plan
(JSON cornerstone conventions: From*/To* pairs, fail-closed parsing; then
INI/TOML/XML/YAML-subset), hash/encoding family (RustCrypto + self-impl
hex/base64 + ByteArrayQ; hashing yes, encryption never), and the downstream
rum-os sketch (RunCommand design, Capability::Process reservation).

## 2026-07-06 — brief-137 gated: string-pattern language is now contract

Four inert atoms implemented, StringSplit IgnoreCase, zero-width anchor
semantics unified and documented ({pos, pos-1} spans), empty-match
consistency, Shortest/MatchQ fix; the whole pattern language incl. the
pre-existing undocumented RegularExpression subset cataloged in reference +
new guide chapter with real transcripts; matrix all-W. Workspace 1151/0.
Pipeline-citizenship brief planned as 139 (Random→140, date/time→141).

## 2026-07-07 — briefs 138-139 gated: JSON codec + pipeline citizenship

brief-138 (a067bda): FromJson/ToJson, zero deps, all numbers exact
(decimals→rationals), fail-closed discipline with offset-bearing messages,
guide chapter structured-data.md; reference 536. brief-139 (8c11068):
filter mode on non-TTY stdin, $ScriptInput verbatim data binding,
$ScriptArgs, stream discipline pinned by tests, quiet EPIPE exit; the
config-conversion shell one-liner works end to end. Workspace 1166/0.
Python-stdlib sweep candidates and the TokenizeWith/RumTokens mechanism
(highlighting for dun as rum-package grammars, no grammar zoo) recorded in
TODO. Next: brief-140 Random family.

## 2026-07-07 — brief-140 gated: Random family

xoshiro256**+splitmix64 as a public reproducibility contract (pinned
literal sequences; two CLI runs identical); PRNG builtins Pure; BlockRandom
back with a real backend; EntropySource seam (StdFileIo pattern) with
Capability::Random gating RandomBytes and zero-arg SeedRandom (in-handler
dual-form check); Capability::Process reserved. Workspace 1177/0. Perl-core
/CPAN-River scan recorded (4 net-new candidates + StringForm inertness
audit; ecosystem scans closed). Next: brief-141 date/time.

## 2026-07-07 — briefs 141-142 gated: date/time + hash/encoding

brief-141 (d672a79): exact rational Unix-second timestamps (epoch as rum's
contract), ISO 8601 fail-closed parsing with offset folding, exact calendar
arithmetic, Clock dual-form readers; 1/3-second round-trip pinned. brief-142
prep (d8a957f): sha2/sha1/md-5/crc32fast pinned + cached (first external
deps under the security/format clause). brief-142 (98da945): digests with
KAT pins, self-implemented hex/base64, ByteArrayQ/ByteArrayToString;
hashing-yes-encryption-never boundary in the guide. Reference 563; workspace
1196/0. Next: brief-143 (Python/Perl tier-1 utilities + StringForm
contract), then the TokenizeWith mechanism batch closes the expansion line.

## 2026-07-07 — STDLIB EXPANSION LINE COMPLETE (briefs 135-144)

From the brief-135 diagnostic to brief-144, the scripting stdlib went
506 → 584 builtins, every addition contract-documented (reference drift
enforced) and transcript-verified: collection odds-and-ends, the
string-pattern contract (dark surface cataloged, four atoms implemented,
edges fixed), FromJson/ToJson (fail-closed cornerstone), rum-cli pipeline
citizenship ($ScriptInput/$ScriptArgs/EPIPE), the deterministic Random
family with the entropy capability, exact date/time, digests + hex/base64,
the 19-builtin utilities batch with the StringForm contract, and
TokenizeWith/RumTokens (highlighting mechanism; grammars stay packages).
Workspace 1212/0. Process briefs archived. Parked: multiline string
literals (parser-level), schema helpers, ByteArray depth, compression
codecs, unicode normalization, ipaddress. Next mainline: user's call
(coverage rebuild is the standing debt).

## 2026-07-07 — CODEC FAMILY LINE STARTED

User picked the codec continuation as the next mainline. Plan committed
(docs/dev/codec-family-plan.md): 145 CSV+INI, 146 TOML (datetimes on the
brief-141 exact timestamps), 147 XML (XmlElement symbolic tree), 148 YAML
sane-subset (aggressively fail-closed; full YAML a permanent non-goal),
M1 conversion-matrix closure.

## 2026-07-07 — CODEC FAMILY LINE COMPLETE (briefs 145-148 + M1)

FromCsv/ToCsv + FromIni/ToIni (d2ab264), FromToml/ToToml (aea144d,
offset-datetime folding onto exact timestamps; encode never synthesizes
datetime syntax), FromXml/ToXml (3a22c5e, XmlElement inert head, no-DTD
security stance), FromYaml/ToYaml (4d49c0d, block-only sane subset,
Norway-safe typing, safe-plain encode quoting). M1 (411d684): six-test
cross-codec matrix incl. the JSON→TOML→YAML→JSON grand-tour identity.
Six formats, one value model, all fail-closed with byte offsets, zero new
dependencies. Reference 595; workspace 1247/0. The config-conversion story
is complete: cat a.json | rum-cli -e 'ToYaml[FromJson[$ScriptInput]]'.

## 2026-07-07 — HARDENING COVERAGE LINE STARTED (A-half)

User decision: the coverage-rebuild debt splits — the infrastructure half
(host lifecycle, limit paths, stress corpus) runs NOW as briefs 149-151;
stdlib-breadth coverage is deferred until stdlib growth plateaus (batch
discipline already covers new code). Plan: docs/dev/hardening-coverage-plan.md.

## 2026-07-07 — brief-149 baseline: the net is better than feared

Workspace region 79.00% / line 82.30%. ALL 12 TerminatedEvaluation tags
have direct exercising tests (the batch discipline held). Real holes:
error-mapping tables (SessionRequestError / HostFailureCode), snapshot/
panic lifecycle edges, iterator-spec parsing, the memory-proxy path, and —
highest risk — the suspension continuation serializers at 0% (never-run
deserializers). Contract note: MemoryConstraintExceeded deliberately has no
TerminatedEvaluation mapping (MemoryConstrained consumes it) — to be made
explicit in docs at M1. brief-150 (test-only buildout) dispatched.

## 2026-07-07 — brief-151: the stress corpus earned its keep immediately

Corpus landed (guard-adjacent pins incl. exact MAX_PARSE_DEPTH boundary,
exact-result rewriting workloads, cancel/timeout under load, battered
session, snapshot at scale; 1289/0, 9 ignored) and DISCOVERED TWO real
bugs: a legal 50k-element list pipeline and 100-500-definition
snapshot/restore both die by native stack overflow — kernel-internal
recursion that never reaches the eval-loop poll point where the
native_stack_limit check lives. brief-152 (diagnose + extend the guard)
dispatched.

## 2026-07-07 — HARDENING COVERAGE LINE (A-half) COMPLETE (briefs 149-152)

149 baseline: region 79.00%/line 82.30%; all 12 TerminatedEvaluation tags
already exercised; verified hole list. 150: test-only buildout (+31),
error-mapping tables and the 0% suspension serializers covered (unit +
real suspend/snapshot/resume), focus files reproduced at gate (outcome.rs
21→97.7%, suspension 0→83-92%). 151: stress corpus (+11) — and it caught
two real bugs: per-element native recursion in Select/DecorateBy resume
helpers (died at 2668 elements) and restore_from_bytes evaluating the whole
snapshot as one nested expression (overflow at ~75 defs; silent definition
loss on big main-thread restores). 152: loops for normal progression
(continuations only on InputRequired), restore through the script/form
splitter; bug stubs are real tests (50k pipeline = exact 25000;
500-definition snapshot round-trip); A/B in the noise band. Workspace
1291/0. MemoryConstrained consume-contract now explicit in the guide.

## 2026-07-07 — A-group + B1 + B2 complete (briefs 153-156)

153: downstream builtin-installer hook (reset/clone survival, capability
dispatch on downstream builtins — the rum-fs/rum-os/dun registration seam)
+ DecimalString + guide touch-ups; rust-toolchain.toml pinned at 1.85.0.
154: adoption-cost numbers measured and documented (session 0.09 ms steady,
eval 4.5 us, 6.5 MiB stripped delta, 23 external crates). 155: schema
validation — patterns are the type language (SchemaErrors data + 
ConformSchema defaults/strict-extras; FromYaml→ConformSchema pipeline).
156: multiline string literals \"\"\"/''' (TOML-mirror), formatter-verbatim
round-trip, ' derivative pinned both sides, is_complete aware. Workspace
1315/0; reference 598. Process briefs archived.

## 2026-07-07 — random family + character-class constants complete (157-159)

157 RandomString/RandomPassword, 158 password policy options, 159 named
Protected character-class constants (UppercaseLetters/LowercaseLetters/
DigitCharacters/PunctuationCharacters/Alphanumeric — sharing brief-137's
punctuation definition; new install_system_own_value kernel hook installs
them as system ownvalues excluded from DumpSave/snapshot). Password policy
usage reads Require -> {UppercaseLetters, DigitCharacters}; constants are
Protected so reassignment fails (Set::wrsym). Reference 605; workspace
1339/0. Briefs 153-159 archived.

## 2026-07-14 — HOST-EMBEDDING LINE OPENED: diagnosis + b1 (brief-231)

Line opened per the user's sequencing ruling (diagnosis → concern (b)
dun pure-sandbox/protocol → concern (a) general embedding API,
scenario-first). Diagnosis landed in docs/dev/host-embedding-line.md
(0ff7d15): pure-sandbox gates 1-3 already done (sandboxed profile,
Capability::Interactive, panic isolation); stdout-zero-writes,
crash-only, and cold start (~4.8 ms full process cycle, 50-run amortized
×3) satisfied but unpinned; the two real seams = structured termination
discriminator + request-level memory ceiling. brief-231 (74afadf) closed
the first seam: EvalTerminationReason (9 reasons, .tag() preserves the
exact legacy strings) threaded finalize → EvalOutcome /
SessionEvalResponse → HostEvalSuccess.termination
(HostTerminationReason mirror). In-band TerminatedEvaluation[...]
values/messages byte-identical; user-constructed literals stay
termination=None (provenance). Gate: fmt clean, workspace 1478/0,
scope exact (12 files), no A/B (finalize runs once per top-level form).

## 2026-07-15 — host line b2: request-level memory ceiling (brief-232)

ResourceLimits.eval_memory_limit (default None) armed per top-level
request at the wall-clock-deadline seam; uncaught breach → in-band
TerminatedEvaluation["MemoryLimit"] + General::memlim + b1's
termination=Some(MemoryLimit). A top-level-only reason helper keeps the
shared reason map untouched, so EvaluationData/TestReport memory
renderings stay byte-identical; MemoryConstrained frames min-combine and
still catch in-band. Active-mode enforcement strided (full proxy walk
every 64 armed checks; projected-byte prechecks unstrided).

Two process notes. (1) The FIRST dispatch of 232 was a wasted round and
MY fault: adding a public field to ResourceLimits forces the five
exhaustive literals in runtime/tests/session_config.rs, which was missing
from the MAY list — Codex correctly stopped at the scope stop-loss with a
clean tree. Lesson recorded: grep the mechanical ripple (new public field
→ every exhaustive literal; new enum variant → every exhaustive match)
BEFORE writing a MAY list. (2) The gate's microbench hung twice at the
run-bounded timeout — instrument failure, not a code defect: rum-cli
binds non-TTY stdin as $ScriptInput, so a benchmark invocation without
`< /dev/null` blocks forever. Confirmed both directions (no stdin →
timeout; /dev/null → 0.33 s). Same family as the "feed probes stdin"
lesson from the stdlib line.

Gate: fmt clean, workspace 1481/0, scope exact (12 files). Armed workload
base 3.825 s → cand 0.667 s (−82.6%, interleaved best-of-3) keeps the
stride. Default-path A/B against a CALIBRATED noise floor (base-vs-base,
same binary: median 2.0%, p90 7.4%, max 10.4%): candidate median −3.4%,
worst label +6.0%, ZERO labels above the floor → no regression.

## 2026-07-15 — host line b3: stdout ownership hard line (brief-233)

The kernel already wrote zero bytes to stdout; this brief turned that
from discipline into a mechanical hygiene test (same family as the
std::fs ban): println!/print!/io::stdout/stdout() forbidden in
rum-core/rum-runtime/rum-stdlib library code, excluding /tests/ and
/bin/ (codegen tools own their stdout). Print stays buffered per request
into `prints`; rum-cli's stdout sink is the host's explicit, legitimate
choice (it owns its stdout), and rum-host exposes no sink at all — so an
embedded session's Print can never reach the host process's stdout,
which is exactly what a framed-protocol host needs.

Instrument trap worth remembering: "eprintln!" CONTAINS the substring
"println!", so a naive contains() check flags all 72 legitimate
env-gated profiling lines (12 files). The scan is identifier-boundary
aware. The hygiene test was self-calibrated at the gate — an injected
println! makes it FAIL and name the file; the director reproduced this
independently in a different file (rum-stdlib/src/lib.rs) before
crediting it. A hygiene test that cannot fail is worthless.

Free property from b2, now tested: `prints` is counted by the memory
proxy, so Do[Print["x"], {100000}] under a small request ceiling
terminates with TerminatedEvaluation["MemoryLimit"] + General::memlim
instead of growing unbounded.

Gate: fmt clean, workspace 1483/0, scope exact (4 files). No A/B — tests
and docs only, no evaluation-path code.

## 2026-07-15 — host line b4: isolation + crash-only contract (brief-234)

reset()=build_inner_session already rebuilds the whole Session from
config (dropping every runtime dimension, keeping config-carried state),
so per-request isolation and crash-only were held by construction but
under-tested — existing coverage touched a single ownvalue. Three
lifecycle contract tests now pin: (1) reset clears own/down values,
attributes, options, messages, contexts, the gensym counter, and PRNG
advancement, compared against a fresh reference session (not hardcoded);
(2) reset-between-requests — no definition leak, and the seeded
RandomInteger stream replays identically (isolation reaches the entropy
stream, which is what lets a host pass explicit state as data); (3) an
abandoned poisoned session leaves a freshly built one unaffected (no
process-global mutable state). embedding.md documents the pattern and
the crash-only guarantee. Probe facts confirmed at write time (not
guessed): empty Options render List[]; bare SetOptions needs Options[h]
pre-declared (SetOptions::optnf); Module in Custom` renders Custom`t$1,
rewound to t$1 after reset.

Honest scope call: the OS-level spawn+SIGKILL+no-temp-files integration
test is deferred to the b5 prototype, where a real host process exists —
SIGKILL-safety is already structural (no temp files, enforced by the
b3 std::fs hygiene test), so a heavyweight flaky spawn test here would
add little. Gate: fmt clean, workspace 1486/0, scope exact (2 files),
no A/B (tests + docs only).

b-line status: b1-b4 DONE (termination discriminator, request memory
ceiling, stdout hard line, isolation/crash-only contract). Remaining:
b5 — the rum protocol host prototype (syntax-highlight role) validated
against dun's hosts/check-host.py.

## 2026-07-16 — Embedding line (a) OPENED: scenario design + consumer survey

Concern (a) of the host-embedding line opened per the sequencing ruling
(scenario design first, no bricklaying on instinct). Landed
docs/dev/embedding-api-line.md as the line's canonical doc: three
user-supplied abstract scenario families (Python/AI-infra adapter,
Rust/wasm tooling, tauri typst IDE — dun's ancestor) + a survey of the
four in-tree consumers with findings F1-F8 (rum-cli smuggles host data
through the input-suspension machinery; HostTyped has zero consumers;
the b1 termination discriminator is unadopted — rum-jupyter still
string-matches TerminatedEvaluation["Cancelled"]; HostFailureCode::Exit
drops the exit status; streaming prints and script/file eval exist only
below the contract). USER RULINGS: dogfood = stance A (in-tree trusted
hosts port onto rum-host; rum-cli first with a microbench A/B hard
gate); brief queue signed off as a1 contract-honesty → a2 trusted-host
surface + dogfood port → a3 typed bridge out → a4 typed bridge in →
a5 named profiles. Also recorded (Q&A): fs/os/net future attach
architecture needs no new invention (register_builtin/installer +
capability gate + provider seams), with three design prerequisites —
the Capability u8 bitmask is FULL (a Network variant forces a widening),
blocking syscalls do not observe cancel, stateful handles vs
snapshot/reset.

## 2026-07-16 — brief-236: host failure-classification honesty (embedding line a1)

Three contract-honesty fixes, host-layer only (zero kernel changes):
HostFailure.exit_status carries the Exit[code] status (was dropped at
the outcome.rs mapping — a contract host could not implement Exit[7]);
HostFailureCode::requires_reset() publishes the reset classification
(poison truth verified from session.rs: only the guard's caught-panic
path sets poisoned, so exactly Panic|Poisoned require reset) and
rum-jupyter's hand-rolled copy now delegates to it; rum-jupyter adopts
brief-231's HostEvalSuccess.termination field, killing the
TerminatedEvaluation["Cancelled"] string-match — deliberate behavior
fix: a program whose ordinary result VALUE is that literal is no longer
misclassified as a cancelled failure (provenance-tested field), pinned
by a new adapter test. Gate: scope exact (7 files, all MAY-listed),
fmt clean, workspace 1488/0 reproduced, key claims reproduced by name
(Exit[7] -> Some(7), reset truth table, literal-value success). No A/B:
failure-path metadata + adapter classification only, no eval hot path.

## 2026-07-16 — brief-237: trusted-host contract surface (embedding line a2a)

Four additive HostSession methods lifting existing Session capabilities
to the embedding contract (zero kernel changes; every method = guarded
forward + existing outcome mapping): set_print_sink/clear_print_sink
(EXCLUSIVE streaming — prints buffer stays empty while a sink is
installed; sink does not survive reset, documented alongside the
cancel-handle invalidation), eval_script (multi-form, last-form value +
termination, aggregated messages/prints), eval_file (reads through the
FileIo provider; provider-less/sandboxed fails as Other), and
queue_input_response (the intended batch-input channel, replacing
rum-cli's input-suspension smuggling). Status quo pinned, not changed:
script suspension surfaces as InputRequired but is NOT resumable —
resume_with_input fails with Other and remaining forms are not
evaluated (a documented seam for a future consumer-driven brief).
Gate: scope exact (3 files), fmt clean, workspace 1494/0 reproduced
(+6 contract tests, each rerun by name). No A/B: additive host facade
only, no kernel code touched. Next: a2b (rum-cli dogfood port,
microbench A/B hard gate).

## 2026-07-16 — brief-238: rum-cli dogfood port (embedding line a2b)

rum-cli's non-interactive paths now ride the rum-host contract,
byte-preserving (27 frozen CLI tests, zero expectation changes): eval /
eval_script / eval_file via HostSession, streaming prints via the
contract sink (EPIPE -> cancel_handle), batch input via
queue_input_response (the argv/stdin smuggling is gone), exit status
via HostFailure.exit_status, message/failure rendering rebuilt
byte-identically from HostMessage/HostFailure. --parse/--format go
straight to rum-syntax (no session construction at all). The REPL
branch keeps a raw Session until a2c (marked in source). The port
exposed and fixed two contract gaps: HostValue.text is now
CanonicalHoldFormTransparent (canonical style with one top-level
HoldForm peeled — the CLI's rendering; jupyter text/plain inherits the
WL-correct display), and HostFailure.syntax_rendered carries the full
rendered diagnostic (Syntax failures only). Process note: the first
dispatch stop-lossed on a brief error (I wrote the expected rendering
as infix "1 + 1" from memory; the canonical truth is "Plus[1, 1]" —
brief revised, expectation values must be probed, not recalled). The
second dispatch stop-lossed on the Part C A/B gate reporting 5
regressed labels; the director's independent 4-pair INTERLEAVED re-run
overturned it: all 5 suspects cleared (best-of-3: literal-string -4.5%,
set-ownvalue -2.8%, block-local -2.1%, symbol-ownvalue-read +0.2%,
downvalue-pattern-hit +0.9%), median delta 0.00%, and with-local's
apparent +14% was label bistability (the same candidate binary observed
at 0.85ms and 0.97ms levels across sessions; pair 4: base 0.000863 vs
cand 0.000865). Lesson pinned: A/B rounds must be interleaved (B C B C),
never sequential blocks — a brief allowing baseline reuse invites
time-drift artifacts. Gate: scope exact (7 files), fmt clean, workspace
1495/0 reproduced, raw A/B data under /tmp/rum-238-gate/.

## 2026-07-16 — brief-239: rum-repl dogfood port (embedding line a2c) — a2 arc CLOSED

rum-repl rides the contract: main loop on is_complete (three-state:
Complete/Invalid submit to eval — syntax failures render via
syntax_rendered — Incomplete accumulates), commands routed per the user
rulings (:lower → new HostSession::lower_dump, diagnostic-grade with an
explicit no-stability-promise rustdoc; :messages → new
HostSession::message_catalog reusing HostMessageTemplate — the
MessageCatalogEntry fields map one-for-one, zero new types;
:parse/:format straight to rum-syntax; :complete/:inspect on the
existing contract). rum-terminal became the contract-type render-twin
crate: render_host_message(s)/failure/eval_response/inspect/
message_catalog, byte-identical (frozen REPL/CLI tests are the oracle);
all 7 runtime-typed renderers deleted with grep-verified zero
consumers. rum-cli's dual session construction unified away. Probed
facts: attribute_count/message_count equal len() (no truncation, no
field additions needed). Dependency payoff: rum-cli and rum-repl no
longer depend on rum-runtime or rum-stdlib AT ALL; rum-terminal depends
only on rum-host — the in-tree hosts are pure contract consumers, which
was the point of the dogfood ruling. Cargo.lock updated for the shrunk
dependency graph (Codex correctly left it out of its allowlisted tree;
the director folded the ripple into the commit). Gate: scope exact (11
files + lock), fmt clean, workspace 1498/0 reproduced, new tests rerun
by name. No A/B: REPL is not the perf vehicle; --eval/script paths
untouched. The a2 dogfood arc (a2a surface, a2b CLI, a2c REPL) is
CLOSED. Next: a3 (typed value bridge, out).

## 2026-07-16 — brief-240: typed value bridge, out (embedding line a3)

HostTyped widened with three documented variants: Bool (the system
symbols True/False now surface as Bool rather than Symbol — deliberate
convenience mapping, no in-tree consumer depended on the old shape),
Association (ordered key/value pairs via the kernel_api association
helpers — no hand-rolled shape detection), and Rule (immediate,
two-argument). Honesty rule pinned by tests: an association containing
a delayed entry and RuleDelayed both stay Other — a typed data view
must not present an unevaluated right-hand side as a value. Depth-cap
discipline uniform across all structured variants. First real consumer
lands with it (killing the F2 zero-consumer status): jupyter execute
results now carry a conservative application/json MIME entry iff the
typed tree is fully representable (Integer/Str/Bool/List/string-keyed
deduplicated Association; BigInteger/Rational/Symbol/Rule/Other or a
depth-capped tail reject the whole tree), with the three existing MIME
entries byte-unchanged (user-approved wire addition 2026-07-16). Gate:
scope exact (6 files), fmt clean, workspace 1506/0 reproduced (+10
tests, all rerun by name), diff reviewed (delayed check precedes entry
conversion; exhaustive match in json_from_typed lists every rejected
variant). No A/B: typed conversion runs once per top-level outcome
mapping; microbench in-process loops never reach it. Next: a4 (typed
value bridge, in) — the line's last core item, then a5 (named
profiles).

## 2026-07-16 — brief-241: typed value bridge, in (embedding line a4)

The input half of the typed bridge: expr_from_typed converts HostTyped
into kernel expressions (depth guard shares MAX_TYPED_DEPTH=64 — trees
under the cap sit far below the documented deep-drop danger, no Drop
machinery touched; Other rejected as an output-only lossy view;
BigInteger/Rational strings validated, zero denominators refused,
rationals normalize so 2/4 round-trips as 1/2) and
HostSession::call(head, args) is the injection-safe invocation path:
head and Symbol arguments resolve through the session's public
resolve_symbol_name — short names hit System builtins exactly as
source text would, fully-qualified names pin both contexts — and a
hostile string argument ("], Print[666], x = " …) round-trips as pure
data (pinned). One kernel seam, authorized by the line's ground rules:
Session::eval_expr_request evaluates an already-built Expr through the
full request protocol (checkpoint/messages/termination), implemented
by extracting eval_request's post-lower tail into a shared helper;
eval_request behavior frozen and verified. A/B story: Codex ran the
newly-mandated interleaved procedure and correctly stop-lossed on 3
labels exceeding their per-label base spreads — but those spreads
(1-11µs, three-sample max-min) were razor-thin artifacts; the
director's independent interleaved re-run measured the same labels'
true cross-round spreads at 5-15% with deltas of +0.6~0.9%, median
delta 0.00%, bidirectional scatter (fastest label -3.8%), two
independent interleaved sessions agreeing. Verdict: no regression;
lesson refined — a FAIL verdict needs the label to clear the WHOLE
FIELD's |delta| distribution, not a three-sample per-label min-max.
Gate: scope exact (6 files), fmt clean, workspace 1519/0 reproduced
(+13 tests, call suite rerun by name; Codex had stopped before the
full suite per its stop-loss — the director completed it). Next: a5
(named profiles), the line's last queued item.

## 2026-07-16 — brief-242: named profiles (embedding line a5) — LINE COMPLETE

hermetic() and scripting() land as named contracts (user-ruled minimal
set): hermetic shares sandboxed()'s implementation but carries its own
determinism promise (same input → same output; fixed-seed PRNG replays
— the stateful↔pure line's dividend: RandomInteger is Pure, only true
entropy is Capability::Random, so the sandbox was already mechanically
hermetic); scripting() names default()'s trusted-local semantics and
rum-cli adopts it (redundant provider hand-wiring gone; the director
removed the leftover unused imports post-gate). The drift-proofing is
the real deliverable: each named profile is pinned by an
all-8-capability probe test with an exhaustive Capability match — a
ninth variant (e.g. a future Network capability, which also forces the
u8 bitmask widening) fails compilation until every profile takes an
explicit stance. Gate: scope exact (4 files + the import cleanup), fmt
clean, workspace 1523/0 reproduced, all 4 new tests rerun by name. No
A/B: config constructors only. One watch item: a rum-jupyter native
ZeroMQ ordering flake surfaced once in Codex's first workspace attempt
(passed in isolation and in two full reruns; also seen once around
brief-165 — if it recurs, brief it).

**EMBEDDING API LINE (concern a) COMPLETE — briefs 236-242, opened and
closed 2026-07-16.** Scenario design (three user-supplied abandoned
downstream concepts) + in-tree consumer survey (F1-F8) drove five
gated increments. Structural endstate: rum-cli and rum-repl are pure
rum-host consumers (zero rum-runtime/rum-stdlib dependencies),
rum-terminal renders contract types only, HostTyped is a bidirectional
injection-safe bridge with a real wire consumer, and the contract
carries the full trusted-host surface plus three capability-pinned
named profiles. The dogfood ruling (stance A) did the work its
rationale promised: every gap was pulled by a real consumer, none
invented.

## 2026-07-17 — briefs 243-253: cleanup + message-registry line — LINE COMPLETE

One day, eight gated briefs, all four queue items closed (user ruling
2026-07-17 opened the line; the same ruling closed the parked pools
WON'T-DO). (1) The message-tag registry arc: inventory (brief-243: 114
push_message sites, 144 tags, 966 closed pairs, live probes proving
Messages[Set] empty), mechanism (245: BuiltinSpec.messages + the
four-surface merge — user definitions win, output shapes frozen), fill
sweeps (246-248: kernel 304 + stdlib 433 + 214 pairs, reconciled
exactly against the inventory; one ledger row proven unreachable at the
source), docgen pairing (249: the reference renders Messages blocks
from the registry — 951 rows, additions-only diff), carriers (252, user
ruling: General 12 pairs + UpTo 2 as inert Protected specs; reference
615 entries; the first dispatch's stop-loss caught an unverified
Take[..., UpTo[...]] expectation — Take/Drop UpTo support recorded as a
WL-conformance follow-up candidate), and enforcement (253: a
debug-assertions check at the push_message seam, declared-or-exempt
rule shaped by two correct stop-loss rounds; zero registry gaps at
landing). (2) vm_rhs_template DELETED (244): the opt-in experiment's ON
path crashed the guard-adjacent recursion test; removal A/B clean.
(3) rum-jupyter signature.rs ported onto Hmac<Sha256> (250) — the
repository's last hand-rolled crypto gone; the ZeroMQ ordering flake
did not recur. (4) rum-cli feature forwarding (251): a four-crate
chain makes the CLI digest-free under --no-default-features with
defaults byte-equivalent. Tooling: the dispatch sandbox now defaults
to workspace-write (danger-full-access reserved, by env override, for
sample-profiling briefs — the original 2026-06-21 reason); the one
observed friction is loopback-bind denial on rum-jupyter's transport
tests, covered by the out-of-sandbox authoritative gate. Cold backup
rum-2026-07-17-dc9181a.bundle verified. Next: the wl15/wfr design
pass (ACTIVE PLAN item 5).

## 2026-07-17 — wl15/wfr pass batch 1: Take/Drop/TakeDrop UpTo (brief-254)

The wl15/wfr design pass (ACTIVE PLAN item 5) opened with the mechanical
batch: `UpTo[n]`/`UpTo[Infinity]` sequence specs for Take/Drop/TakeDrop
(clamp-to-length, any spec position, multi-level free via per-spec
parsing). Malformed contents reuse the brief-252-registered UpTo::innf/
argx carriers. Gate correction: the innf trigger was narrowed to
definitely-invalid contents (negative integer or rational) to mirror the
kernel Part-span rule — symbolic (`UpTo[x]`) and oversize integers stay
on the silent inert path. Workspace 1523/0; probes reproduced at the
gate. Commits cd81665 (brief) + 5a2ef4a (impl). Batches 2 (hold design)
and 3 (structured exceptions) proceed design-first.

## 2026-07-18 — wl15/wfr pass batch 2 (hold half) COMPLETE bar 2c

Batch 2 landed the SubValuesHoldAll line: 2a mechanism (brief-255,
8186536 — new attribute bit 1<<15, compound-head args prepared as
HoldAll via the existing base-symbol walk, symbol-head calls
untouched); 2b-I mutation operator forms (brief-256, acaf1f3 — nine
compound-assignment heads AddTo/AppendTo/ApplyTo/AssociateTo/DivideBy/
KeyDropFrom/PrependTo/SubtractFrom/TimesBy adopt the attribute + a
mutation wrapper family; unary captures quiet-inert; two brief
revisions on file ownership); 2b-II schema operator forms (brief-257,
476cf19 — ConformSchema/SchemaErrors switch to special_with_hold, raw
schema captured, value evaluation handler-owned, byte-parity with the
direct form incl. its pre-existing validation-time pattern resolution).
Workspace 1543/0. All three A/B gates clean (no same-direction hot-label
regression). Remaining: 2c ReplaceAllUnheld (brief-258). Gate lessons
this batch: file ownership must be opened-and-verified not name-grepped
(brief-256 two stop-losses); equivalence contracts pin A==B side-by-side
not "both have property P" (brief-257 one stop-loss). Both recorded in
[[brief-may-list-must-cover-mechanical-ripple]].

## 2026-07-18 — wl15/wfr batch 2 CLOSED: ReplaceAllUnheld (brief-258)

2c landed as a separate builtin (ruling C1, 48e909f): ReplaceAllUnheld
evaluates its target first (mirror ReplaceAll), then traverses the
evaluated tree with a structural descent mask that skips held argument
positions. The mask reads the ORIGINAL pre-rewrite head attributes and
honors compound-head SubValuesHoldAll (brief-255 consistency);
ReplaceAll's own path is byte-unchanged (the mask branch short-circuits
on the source discriminant). Operator form via one line in
rewrite_operator_symbol. Workspace 1547/0; A/B 3-pair interleaved with
replaceall-simple as the hard veto — -2.53% (faster), tier flat-to-
better, inside the control noise floor. Codex correctly stop-lossed on
a missing stdlib_attributes.rs vocabulary-mirror line (new-builtin
ripple I omitted from MAY); it left the complete correct implementation
in the tree, so the director added that one mechanical line and ran the
full gate rather than waste a re-dispatch. Lesson recorded in
[[brief-may-list-must-cover-mechanical-ripple]]. BATCH 2 (the hold half)
now fully closed: 2a SubValuesHoldAll + 2b-I mutation forms + 2b-II
schema forms + 2c ReplaceAllUnheld. Remaining in the wl15/wfr pass:
batch 3 structured exceptions, pending user green light on E1-E6.

## 2026-07-18 — wl15/wfr batch 3 CLOSED + THE LINE COMPLETE: structured exceptions (brief-259)

3a core landed (42a2476): a folk structured-exception system with
observably-separate channels from classic Throw/Catch. A new
EvalError::ExceptionThrown variant carries the Exception[type, payload]
object; Catch's match arms never mention it, so the type system makes
cross-catching impossible (director sized this via cargo check: exactly
one non-wildcard EvalError match breaks, the top-level mapper). Five
heads register in stdlib and dispatch to kernel eval via
ControlDataBuiltin (Exception inert; ThrowException/CatchExceptions/
RegisterExceptionType/ExceptionTypes active). String type hierarchy in
a session-field registry mirroring message_definitions (snapshot
round-trip covered). Ancestry dispatch = handler list-order priority ×
nearest-ancestor, suspension-safe via the same matcher Catch uses;
no-match re-raises. E4 uncaught → in-band Failure["UncaughtException",
<|Type,Payload|>] at finalize_top_level_eval_value + a nocatch message.
E5 unregistered-throw fails closed to a root exception. E6: WithCleanup
runs cleanup on unwind (free via its wildcard path), Enclose passes
exceptions through. Workspace 1554/0; reference 621. Two brief revisions,
both correct Codex stop-losses on my brief omissions (stdlib file
ownership for the control family; the kernel-eval/stdlib-registration
ControlDataBuiltin split). 3b = the evaluation.md §Structured Exceptions
guide section (director-written, examples measured on the release CLI).

**THE wl15/wfr DESIGN PASS IS COMPLETE** (ACTIVE PLAN item 5, briefs
254-259): batch 1 Take/Drop UpTo; batch 2 the hold half (SubValuesHoldAll
+ mutation/schema operator forms + ReplaceAllUnheld); batch 3 structured
exceptions. The cleanup + message-registry line (item 5 was its last
open item) is now fully closed.

## 2026-07-18 — catalog file-size splits (briefs 260-261)

Cleared the split-plan-range (20-35KB) catalog files left by the
message-registry fill. brief-260 (e48d12f): kernel builtin_catalog/
namespace.rs (26.0KB) → namespace/{variables,contexts,symbols}.rs
(1.4/10.4/14.4KB), Vec aggregator concatenates in order. brief-261
(fdf2d6a): stdlib strings.rs (27.5KB) → strings/strings_text/strings_match
(STRING_CORE/TEXT/MATCH), structure.rs (25.4KB) → structure/
structure_ordering, associations.rs (21.0KB) → associations/
associations_transform; sibling files preserve super::message_templates
imports; seven sub-consts registered across catalog/mod.rs + lib.rs +
tests.rs in preserved family order. Both briefs: pure reorganization,
reference byte-identical (the master gate — docgen reads registered
specs and sorts by name), workspace 1554/0, all resulting files under
15KB. rules.rs (17KB) is warning-range only, left as-is. Remaining
over-20KB: stdlib_attributes.rs (28KB) — a flat alphabetically-sorted
attribute mirror with a binary_search_by consumer, so splitting is a
lookup-mechanism change with no family boundary; surfaced to the user
as split-with-adaptation vs. accept-as-debt.

## 2026-07-20 — Sealed scoping line (d) COMPLETE (briefs 268-271)

The lazy + sealed-scoping ACTIVE PLAN is fully closed. (d) landed in four
steps: brief-268 grounding (REFUTED the design's original Module-gensym
substrate — Module locals are name-mangled and source-reachable; the real
seal = fresh bit-62 identities kept OUT of symbol_generations, resolver
byte-untouched), brief-269 mechanism spine (DefinePackage three-tier
registration, suspension-safe install, 12 reachability tests, reference
637), brief-270 hidden-by-default introspection (one predicate + one
Diagnostic-gated reveal across Names/Contexts/Definition/Information/
catalogs/completion/Save-DumpSave; 17 dual-variant tests), brief-271
snapshot-by-re-registration + re-link (RestoreSealedReferences restore-mode
marker, orphan refusal, fallible snapshot_to_bytes, byte-parity for
sealed-free sessions; reference 638). Close-out: guide §Sealed Packages,
README divergence section (user ruling 2026-07-19: the dual dividend —
real encapsulation + persistence in one mode, dissolving the strict/stable
split — is publication-narrative material). Workspace 1630/0. Gate
corrections this line: brief-271's undeclared process-global identity
allocator reverted to session-scoped. A/B method note: the microsecond
list-tier labels proved layout-lottery-sensitive (±60%, bidirectional,
closed by brief-271's reversal of brief-269's movements).

## 2026-07-20 — Row-workload pure line (DB-prep) COMPLETE (briefs 272-274)

User ruling opened and closed the line in one day: pure builtins only,
preparing the "table = List of Associations" workload for future
downstream database applications. Audit (F1-F5) then three gated briefs:
272 JoinAcross (hash-based Inner/Left/Right/Outer joins, key spec =
key | Key | rule | list; right-wins collisions; unmatched rows bare;
reference 638->639); 273 SortBy multi-key tie-break form (literal-List
spec -> evaluated key tuple, lexicographic + stable) + MaximalBy/
MinimalBy (reference 639->641; gate fix: ASCII-sort violation in the
attribute-mirror insert, no sortedness assertion exists - debt note);
274 kernel named-slot fill from Association arguments (Lookup-
consistent Missing on absent key; microbench A/B noise-floor clean).
Endstate: workspace 1644/0, reference 641, guide syntax.md documents
named-slot fill. Not pursued: Query head (Dataset subsystem, out of
folk scope); parser wrinkle "#1 &[x]" without parens parked.

## 2026-07-20 → 07-21 — matcher respec, debt clearing, REPL/trace (briefs 275-288)

Endstate: workspace 1675/0, reference 642, zero files over the 35k
architecture-debt line.

- **Matcher OneIdentity/Optional + Flat respec (275-279).** WK-derived
  `#[ignore]` tests retired as policy fossils and re-pinned to rum's own
  measured behavior; non-`OneIdentity` `Flat` singleton wrapped candidates
  implemented, closing a real gap (`MatchQ[g[1,2], g[a_g, b_]]` was False
  though `g[g[1], 2]` flattens to `g[1,2]`). The `optional.rs` collect-all
  omission arm was first-match-shaped and under-collected; fixing it is not
  `OneIdentity`-gated, so built-in `Plus`/`Times` recovered dropped rows.
  2^n candidate growth accepted per user ruling (no pruning).
- **Debt clearing (280-281).** `stdlib_attributes.rs` split along its ASCII
  name ordering into four first-byte chunks, lookup still const-time;
  `rum-host/src/tests.rs` (68,639 bytes) split by responsibility into six
  files, proven a pure move by name-set equality.
- **REPL/trace (282-288).** REPL rendering frozen with full-output snapshots
  (`contains` 48 → 0); `:trace`; `TraceEvent` switched from `String` to
  `Expr`, deleting a render→parse round trip and cutting tracing overhead
  from 5.5-5.8× to 2.4×; `TraceDetail` (lossless, names which overlapping
  definition fired, key `RuleLeft`); `:trace --rules`.
- Re-probed and deliberately not actioned: symbolic `Plus`/`Times` like-term
  detection confirmed superlinear on distinct terms from roughly n ≥ 400,
  but the entry's own revisit trigger (a real symbolic-algebra workload)
  is still unmet.

## 2026-07-29/30 — incremental container update + positional Part assignment (briefs 291-300)

Two arcs, both opened by design-only briefs under the plan-first workflow.
Origin was a rum-hol feasibility question about log-replay cost, which led to
measuring `assoc[k] = v` and finding it O(n^3).

plan-291 (briefs 292-296): microbench labels; SharedArgs COW + cache-proof
primitives; flat AppendTo fast path; Part spine; Association direct
canonical-Rule mutation. AppendTo O(n^2)->O(n) (6.4x), Part spine 13.2x,
Association 25.1->0.43 ms (57.9x, shared_expr_alloc 20322->222). Step 6
(bounded key index) deliberately not done.

plan-297 (briefs 298-300): the diagnosis brief found that `lst[[i]] = v` never
worked -- it left the list unchanged and emitted a false "Set::wrsym: Symbol
Part is Protected". Fixed as a feature (88625df), measured (885ec0f), then made
independent of container width (60c4ab4; length quotient 2.87 -> 0.995).

Durable lessons, all learned the hard way in this arc:

- Counters (allocation counts) are immune to machine load; timings are not. A
  video call invalidated an entire timing pass while every "did the mechanism
  fire" conclusion still stood. Since brief-294 the rule is: report counters
  first, and if they do not move, do not quote a time.
- This suite has a per-run whole-run offset moving all labels together by a few
  percent. Two runs of the identical script read -4.07% with 7/38 up, so a
  lopsided direction split is not evidence. Alternating order does not cancel
  it.
- Never normalize against the suite median in a lane of successive
  optimizations: it drifts cumulatively (a sentinel read +21.9% across four
  steps purely because optimized labels dragged the median down). Use named
  untouched labels, or within-run quotients.
- Cross-session label comparison is unreliable even with paired ratios: the
  same comparison read -4.96% across sessions and -0.16% within one.
- Every performance probe needs an assertion that the work actually happened.
  The `lst[[i]] = v` numbers measured a failing rebuild-then-reject path for
  several rounds. The probe contained a calibration point that was never read,
  and the scaling check did not catch it because the failing path is genuinely
  O(m) -- the number scaled correctly while measuring the wrong thing.
- Plan-first paid off immediately: Codex's design brief refuted three of five
  findings on 291 (the call-stability cache as hidden third owner; "only
  ApplyTo runs user code" false for all eleven compound assignments;
  Association O(n^3) not O(n^2)), all confirmed at the gate, and on 297 found
  the feature gap that had been mis-diagnosed as a performance problem.

## 2026-08-22 → 08-23 — post-v0.1 safe-Rust algorithmic performance sweep

The first repository-wide audit for avoidable O(n^2)-and-worse work after
`v0.1.0` landed as 53 commits after the prior documentation checkpoint
`2c8b824`: 50 performance changes, two correctness fixes found while tracing
cost shapes, and one test-environment fix. `rum-core` and `rum-runtime` remain
zero-`unsafe`.

The accepted changes fall into six groups:

- resumable predicate, collection, traversal, tensor, fold, sort, equivalence,
  and grouping loops now keep cursors/accumulators instead of rebuilding or
  recursively cloning continuation state;
- wide collections, associations, schemas, arithmetic groups, definition
  candidates, codec dispatch, and exact replacement rules use adaptive indexes
  while retaining allocation-light small-input tiers;
- polynomial transforms batch canonicalization and use sparse/indexed term,
  variable, factor-multiset, and containment machinery; this optimizes the
  existing polynomial kernel but deliberately does not expand `Factor`'s
  documented algorithmic scope;
- `ReplacePart`, `ReplaceAll`, batched association selection, DownValue/UpValue
  construction and lookup, traversal, and `Module` escape analysis no longer
  repeat suffix copies or linear closure scans;
- string matching/search, reversible suffix matching, entity scanning, wide
  sequence edits, `Drop`, and bounded `Nearest` selection were linearized or
  indexed where the previous wide path rescanned prefixes/suffixes;
- the final P0 tranche landed as `c73df5c` (traversal suffix state), `97873ee`
  (`Module` escape closure), and `d97abd7` (wide UpValue symbols).

Two correctness defects surfaced during the audit rather than being hidden by
performance patches: recursive operator parsing is now depth-bounded
(`28afe35`), and uninitialized `Module` bindings are processed iteratively
(`28b321c`). The nine native ZeroMQ tests now skip when loopback access is
unavailable (`69fde1f`), so ordinary sandboxed workspace tests no longer need
an out-of-sandbox rerun merely to classify that environment.

Correction to the 2026-07-20/21 entry above: its statement that symbolic
`Plus`/`Times` like-term grouping remained an accepted quadratic tradeoff is
historical, not current. Commit `48838b3` added an adaptive group index to all
four additive/multiplicative grouping paths. Inputs stay linear-scan below the
threshold; wide inputs build a `HashMap` after 16 distinct groups when enough
work remains. The old unconditional O(n^2) description has therefore been
removed from `TODO.md`.

Timing discipline was also re-confirmed. Elapsed measurements come from
`AbsoluteTiming`, whose implementation uses Rust's monotonic `Instant`; wall
clock differencing is invalid on a host whose civil clock can slew or step.
Competing CPU load can still invalidate timings even though it cannot make the
monotonic clock run backward. Counters remain useful for proving that a
mechanism fired, but do not substitute for an uncontended timing gate.

Three P1 items remain: the ordered all-`Single` matcher cursor; a shared
`RewriteRulePlan` plus negative-level-only depth computation for `Replace`; and
the bounded `RumTokens` prefix, TOML table-key, and `Reap` tag-bucket scans.
An ordered-matcher candidate has been prepared but deliberately not accepted or
committed while another CPU-heavy workload makes its timing gate unreliable.
`Factor` expansion is separately deferred because a complete
factorizer needs finite-field polynomial machinery, not another local linear-
factor optimization.

## 2026-08-23 — P1 ordered matcher cursor

Commit `d93c859` replaced repeated remaining-vector rebuilds in the ordered,
non-`Flat`, multi-argument matcher with owned pattern/value cursors. The cursor
continues across all-`Single` prefixes, falls back to the existing generic
sequence/optional machinery on the first variable-arity argument, and is
preserved by typed suspension continuations so resumed conditions do not replay
already matched prefixes. The implementation remains safe Rust.

The acceptance gate used independent release builds from the committed base
and candidate source trees. Timing came only from the benchmark program's
`AbsoluteTiming`/Rust `Instant`; the suite's wall clock was used only for its
process timeout record. With loop-style labels scaled 3x and B/C each run five
times, median candidate changes were -17.06% for four arguments, -89.59% for
800 arguments, and -94.74% for 1600 arguments. Same-tier `MatchQ`,
`ReplaceAll`, `Cases`, `Count`, and `Position` sentinels ranged from -1.61% to
-0.18%, so none regressed.

The allocator gate was collected in the normal sandbox with the new
`core-memory-microbench.sh --no-rss` mode because macOS `/usr/bin/time -l`
tries a denied `sysctl` even after its child succeeds. After subtracting the
fresh-process baseline, allocation counts changed only -0.15%/-0.08% at
800/1600 arguments, while allocated bytes fell 8.63% at both widths and peak
live allocator bytes fell 8.27%/8.21%. Rum's expression-memory proxy remained
360 bytes on both sides because it does not account for these Rust temporary
vectors. This is a time-and-space improvement, not a space-for-time tradeoff.

Correctness gates passed `cargo test -p rum-runtime` (674/674) and
`cargo test --workspace` (all non-ignored tests; three existing stdlib stress
tests remained ignored). The remaining P1 order is now: shared `Replace`
planning/depth work, then the bounded `RumTokens`/TOML/`Reap` wide-input scans.

## 2026-08-24 — second algorithmic sweep, crash class, and two cliffs

The post-v0.1 audit continued through briefs 301-321. Beyond the complexity
reductions it set out to make, it closed two defect classes the original survey
had not named, and one of those was found only because a probe was pushed past
the size the plan had specified.

**Native stack exhaustion on wide input was a shipped bug, not a regression.**
`Replace[Range[10000], z -> 0]` and `Cases[Range[7000], _Integer]` aborted the
process; `origin/main` aborted identically. The walkers recursed once per
element from inside a `drive_internal_value` continuation, and the native-stack
guard could not see it: the guard polls in `evaluate`, keyed on
`evaluation_depth`, which these continuation chains never increment. Raising a
limit was therefore not the fix and adding a poll would only have converted a
crash into a refusal — a 50 000-element list is a reasonable input. Thirteen
builtins were converted to explicit loops carrying a `pending` outcome so a
suspension resumes at the same element: `Replace`, `ReplaceAll`, `Cases`,
`Count`, `Position`, `DeleteCases`, `ReplaceList`, `Reap` rendering, the
`UpValue`/`SubValue` dispatchers, `StringReplace`, `StringCases` and
`StringApply`. The boundary is now `GeneratedElementLimit`, which refuses
gracefully.

**Two cliffs, both invisible until a probe crossed them.**

Incremental association writes cost 1.5 s at 16 000 keys and 55 s at 20 000 —
1.25x the work for 36x the time. Every direct write stability-checked a freshly
built `Rule`, inserting a cache entry for an expression that died immediately,
so crossing the 16384 entry cap wiped the whole cache including the proof the
direct path needs; the slow path never re-seeded it, disabling the fast path for
the rest of the session. The table in the brief that opened this work stopped at
16 000 — precisely below the cliff — and it surfaced only because the worker
measured past the specified range.

Installing many UpValues on one symbol took 38.6 s at 20 000 definitions. The
diagnosis refuted all three suspected causes (index rebuild on non-terminal
insert, specificity recomputation in the comparator, metadata prune) and found
the trigger was punctuation: each line ending in `;` lowers to a
`CompoundExpression` wrapper, ordinary calls enter UpValue dispatch before
builtin dispatch, and the wrapper's arguments contain the tag nested inside the
`TagSetDelayed`. A 128-line probe showed `candidates=8128` — C(128,2) — with a
prefilter skip rate of 1000 per mille: every candidate rejected, the whole scan
wasted. The same definitions took 2.54 s with trailing semicolons and 0.03 s
without.

**Measured outcomes.** Script loading 7.75 s to 0.01 s at 10 000 lines, plus one
parse per form instead of two. `ToToml` 6.9 s to 0.03 s at 8000 entries;
`FromToml` 0.289 s to 0.014 s at 16 000 keys. `StringReplace` and `StringCases`
2.7 s and 3.4 s to 0.017 s and 0.018 s at 8000 strings. Association writes
675 s to 0.08 s at 32 000 keys, across the cliff fix and the write index.
`KeyTake`/`KeyDrop`/`KeyDropFrom` roughly 20x at 8000 keys. `ReplacePart`
0.0175 s to 0.0032 s at 2000 rules, and 20 000 rules from a crash to 0.014 s.
`Together` 0.53 s to 0.077 s at 40 addends. UpValue installation 38.6 s to
0.15 s. `Reap` collection 0.129 s to 0.014 s at 8000 tags. Workspace tests went
from 1862 to 1921, always green.

**Method notes worth keeping.**

- *A brief's own measurement range can hide the defect.* The association table
  stopped one step short of the cliff. Probe past the specified range.
- *Three green probes do not establish a universal claim.* brief-305's design
  rested on "balance open implies an incomplete parse error"; the spot-checks
  passed only because juxtaposition rescues adjacent operands, and real grammar
  conflicts die before EOF. The brief was withdrawn intact — the working tree
  was byte-identical — and replaced by a parser-side design whose equivalence
  follows from prefix determinism instead of sampling.
- *Counters settle what stopwatches cannot.* Whether a write mutates in place is
  a yes/no question with a load-immune answer: 65,840,485 allocations became
  20,020, about one per write. A probe also demonstrated the ownership trap
  directly — a weak witness alive across `make_mut` forces a copy, dropping it
  first keeps the allocation.
- *A survey's structural reading needs a probe before it becomes a plan.*
  `DownValue` dispatch was listed with `UpValue`/`SubValue` on structure alone
  and measured fine. `StringCases` was cited as a linear in-tree model, but that
  measurement used a literal pattern routed elsewhere; its rule form had the
  same defect. Conversely `KeyDropFrom`, named without a probe, was confirmed.
- *Semantics-preserving is not the same as merging.* `ReplacePart` rules apply
  sequentially, each to the result of the last, so merging targets collected
  against the original tree changes results silently. `Together`'s obvious
  deferred-normalization fix would have made identical denominators slower;
  pairwise merging kept the per-step normalization and only changed operand
  sizes.

Two items from the original P1 list remain and are recorded in `TODO.md`:
`RumTokens` offset conversion and the shared replacement plan for `Replace`.

## 2026-08-26 — third sweep: crashes, chain lowering, and callback ordering

Two independent re-surveys produced about thirty items, worked in batches. Six
batches ran; one turned out to be already fixed. Every batch below was gated by
reproducing its evidence directly rather than accepting the report's numbers.

**Closed.** Loop and `Module`-initializer drivers made iterative — a
20 000-iteration `While` had terminated the process, which is a correctness
defect before it is a performance one. YAML and TOML decoders bounded to match
`FromJson`, so all three accept 256 nested collections and refuse the 257th
with a byte offset. Left-associative chain lowering rebuilt over five steps:
all sixteen affected operators overflowed the stack at 20 000 links, and they
now fold iteratively, nine into n-ary calls, under a 4 096-link ceiling proved
on a 2 MiB embedding stack through parse, lower, format and drop. Resource
budgets made symmetric across families. User-callback ordering moved onto one
resumable merge engine: `Sort` and friends were quadratic on every input, and
`KeySort` overflowed the stack at 110 keys.

**The instrument lied four times, and that is the durable part of this entry.**

1. *The microbench never closed stdin.* `run-bounded.py` launches what it is
   handed, and rum-cli binds non-TTY stdin as `$ScriptInput` when given a
   program argument, so an inherited pipe that never reaches EOF left it
   waiting until the timeout. A run therefore either finished in four seconds
   or burned its whole timeout and produced a timings file containing only a
   header — with no mention of stdin, so it read as a loaded machine. A worker
   spent forty minutes polling `uptime` looking for a quiet window that would
   not have helped, and every A/B before `dd4bdefc` was exposed to this.
2. *A probe measured the wrong builtin form.* rum's custom equivalence test is
   a positional argument, not a `SameTest` option. `DeleteDuplicates[list,
   SameTest -> f]` passes a `Rule` where a function belongs, so every
   comparison fails and nothing is deduplicated — which is why an all-identical
   list appeared to cost the same as an all-distinct one, and why "SameTest
   never exits early" reached a brief as a finding. With the supported syntax
   the early exit is obvious: 399 predicate calls versus 79 800 at n=400.
3. *A trace probe closed over the wrong scope.* `(trace = Append[trace, #]; #
   &)` binds `&` to the final `#` only, so the assignment ran once, outside the
   function, and the trace held a bare `Slot`. It would have supported a
   confident claim that key evaluation count had changed.
4. *A survey finding was a fossil.* The "Association 4-to-5" and "65 536-key"
   cliffs did not exist any more; `c8d34876` and `dc5f6698` had closed them
   earlier in the same sweep. Eight operations measured flat.

The common root is that none of those probes carried an assertion that the work
they claimed to measure had happened. A call count, a result length, or a
`Depth` check would have caught all four immediately. That requirement is now
written into the briefs.

**A regression I introduced, and what it teaches about gates.** Charging nested
iterator counts as a running product was right for `Table[Table[…]]` and wrong
for `Do[Table[…], {200}]`: a `Do` discards each value, so 200 sequential rounds
of 1000 elements are not 200 000 simultaneous ones. The propagation rule is now
whether the enclosing construct *retains* what the inner one produces. The gate
that missed it had tested that separate statements do not share an accumulator
— the same question asked only from the bypass side. Asking it from the
false-refusal side was a matter of enumerating enclosing-by-inner construct
combinations, which is now how these are gated.

**Two workers disagreeing with me was worth more than either agreeing.** The
plan for the chain work refuted three of my premises, including that `:` cannot
be flattened because its lowering is stateful per link — a detail I had not
seen. The plan for the ordering work refuted my `SameTest` finding outright and
found the `KeySort` crash next to it. A worker stopping clean on a `Scope`
omission — the merge engine sits behind a private `mod`, and the MAY list named
the engine file but not the module declaring it — cost one redispatch and no
wrong code.

**Deferred with evidence:** parentheses are transparent to comparison-chain
collection, so `1 == (1 == 1)` is True while `1 == True` stays unevaluated; the
rhs half is fixable, the lhs half shares one AST with the unparenthesized form.
Product propagation is decided from construct nesting rather than data flow, so
`Table[Length[Table[…]]]` is refused conservatively. Both are recorded in place
— the first as an annotated golden row — because a defect written into a test
without comment becomes an expectation the next reader will defend.

## 2026-08-27 — batch 7 re-probe, and the two defects it uncovered

Batch 7 of the post-`v0.1.0` sweep was supposed to be six performance items.
Re-probing them first — the rule this project adopted after two earlier
findings turned out to be already fixed — is what made the batch worth
running, because two of the three most valuable results came from probes
written to measure something else.

**The batch itself, all six re-probed and closed.** `ValueQ` at 8 000
definitions went from 332.6 us to 0.76 us per call and, more importantly,
stopped scaling with session history at all. Association `Part` reads went
from 96 101 allocations to 14, independent of width; positional reads are
exactly flat across width, keyed reads sublinear. `StringTrim` with a literal
list got the fast path its single-literal sibling already had, 31.6x and
36.1x. Symbol ordering stopped allocating two `String`s per comparison with
byte-identical output. The REPL runs one full completeness check per form
instead of one per line. The Jupyter kernel sleeps instead of spinning a core,
and the gensym sweep rearms at a watermark so per-batch cost stops climbing —
+106% by the third batch before, -11% after.

**`Do[f[i] = i, {i, 1, 3}]` never worked.** It installed a single
`HoldPattern[f[i]] :> 3` and said nothing. A clean iterator symbol is bound in
a dynamic own-value frame; ordinary evaluation resolves definitions and then
that frame, but the held-lhs normalizer stopped after definitions — invisible
exactly where the key is built. The decisive experiment was pre-valuing the
symbol: that pushes the iterator onto its fallback branch, which installs a
real definition, and the same code then produces the right keys. The fix is
the normalizer taking the same third step in the same order.

**Right-hand-side `Condition` guards were not implemented anywhere.**
`f[x_] := rhs /; test` stored the whole `Condition` as the body and never
looked at it, so `b[x_] := x^2 /; x > 0` made `b[3]` return `9 /; True` and
`b[-3]` return `9 /; False`. Dedup compared left-hand sides only, so a rule
differing solely by its guard replaced the unguarded one and vanished. Eight
candidate-selecting lanes were affected. `Cases` was the sharpest: it
collected elements whose guard had declined — a wrong answer, not a leaked
wrapper. Repaired lane by lane with one substrate and one accept rule, with a
decline made indistinguishable from a pattern mismatch to every consumer,
including `ReplaceRepeated`'s fixed-point test and `ReplaceList`'s enumeration.

**The lesson that cost the most.** After six green gates on that line, a
guarded `DownValue` still stopped enforcing its guard the moment anything
bumped the attribute revision — any `SetAttributes` in the session, or a
loader-snapshot restore such as `ValueQ`. The reconstruction written in the
first step was correct but unreachable: on a stale revision the fast path
rematched from the lhs alone, succeeded for a simple pattern, and returned
before reaching it. Every gate, and the whole-line regression test, had
asserted the guard *before* the trigger and never after one, so none of them
could see it. The repair put the barrier in the single rematch API rather than
patching a second call site, and the regression test now carries an
attribute-bump dimension. For anything that caches or compiles state derived
from a definition, the test has to act after the invalidation.

**Two instrument failures, both caught by their own guards.** A benchmark row
added for the gensym work emitted `Do::argform` and measured a failing path
until a liveness guard was added asserting the batch actually reaped its
values. And a stop-loss fired on three control labels that had exceeded
floors computed from two baseline runs; three runs each way put the real
same-binary spread at 12.6-26.7% and the movements at -1.72%, -0.01% and
-16.16%. Two runs give one random difference, not a floor.

**Opened, not closed.** Gensyms held only by an opaque continuation frame are
not treated as live, and it is reachable two ways. The `rum-jupyter` ZeroMQ
ordering flake reached its third appearance. Both are recorded in `TODO.md`
with what a fix would have to decide.

## 2026-08-28 — the suspension-frame roots line (briefs 364-375)

The defect `TODO.md` opened after batch 7 is closed. A gensym reachable only
from a suspension frame could be swept while still needed, because the root
walk stopped at `active_stack` and the frames themselves were
`Box<dyn SuspensionFrame>` wrapping arbitrary closures — state Rust cannot
enumerate. The repair replaced that with named frames whose traversal is
derived from their fields, and made forgetting it a compile error rather than
a thing to remember.

**Four defects, not two.** The design brief's re-probe found a matcher binding
table (`SymbolExprMap`) losing its bindings, and a `Block` frame holding a
bare `Symbol` that *leaked* on the following sweep — failing in the opposite
direction. That fourth one settled a design question: a visitor covering only
`Expr` would have fixed the three requested probes and left the leak in place,
so `Symbol` is a root in its own right. All four are verified closed against
plan-364's original probes, which assert the old buggy values and now fail by
asserting the defect.

**Enforcement, not vigilance.** `SuspensionFrame: FrameRootTrace` makes a
missing traversal fail compilation; the frame queue is private so there is one
insertion route; a repository invariant test bans the deleted APIs and any
handwritten traversal outside a sorted two-entry allowlist. Both allowlist
entries pass a two-part test argued per type: a derive is structurally
impossible (`std`, `rum-syntax`), and the type cannot name `Expr` at all
because its crate cannot reach `rum-core`. The honest limit is recorded too —
Rust cannot prove an impl came from a derive across crates, so the allowlist
is the boundary, accepted by user ruling on 2026-08-27 rather than paying for
absolute sealing by moving every stdlib frame into `rum-runtime` and undoing
the crate-rebalance line.

**It paid for itself on the ordinary path.** A closure captures at
construction, before the `Ok`/`InputRequired` split, so every call built a
suspension capture it usually threw away: a width-2003 `Switch` paid 63.8 KiB.
Measured by differential against a lazy-capturing control, that width slope is
now -128 bytes of constant noise. `INTO_VEC_CLONE` could not see the copy at
all — the builtin boundary had already degraded `SharedArgs` to a slice — so
the allocation counter was the applicable instrument.

**Six stop-losses, five of them mine.** A list treated as scope authority on a
line whose purpose is to replace lists with compiler enforcement; an absolute
prohibition with no test for its first legitimate exception; a
stop-on-second-candidate rule whose test was mechanically checkable all along;
and twice, reaching for step 374's frame lifetime before it existed — once in
the cutover's `self.as_ref().clone()`, once in carriers assuming
`try_unwrap` uniqueness while a rooted frame still held an alias. The one
genuinely technical stop was the worker's: a suspension-only merge
representation changed observable comparator progression, so the sort family
waited for the cutover.

**Instrument notes.** A scan for remaining escape hatches reported zero
everywhere including inside `suspension/core.rs`, where eight were known to
remain; the regex had broken on the comma in `Result<Expr, EvalError>`.
Recalibrated against that known-eight control, it then confirmed zero
elsewhere. A scanner reporting a clean result has to be shown finding
something first. Separately, a probe that "reproduced" nothing turned out to
be an `Option<Expr>` field left as `None`, which visits nothing — a probe
needs a value that can actually be seen.

**A/B read in both directions.** Step 372's labels all improved and step 373's
all regressed, and both were reported as no regression: the untouched controls
moved as far or further each time. Step 373 had the stronger evidence — the
sort family had zero diff and still moved 7%, and code that was not touched
cannot have been affected even indirectly.

Endstate: workspace 2163/0, zero build warnings, no production file over 35k,
and a first occurrence of a call-stability-proof flake recorded in `TODO.md`
without adopting the worker's explanation for it.

## 2026-08-28 — the assignment-target normalization line (briefs 376-383)

Opened from three items `TODO.md` had recorded as batch-7 leftovers. A
re-probe found one of the four already fixed four days earlier and never
retired from the record, and the design brief then found three more defects of
the same family. The line closed eight defects in total across two repair
families.

**Family A (brief-377)** — thirteen mutation heads did not evaluate a `Part`
selector, so `kk = 1; lst[[kk]] += 5` reported `Part::pkspec1` and changed
nothing while `lst[[kk]] = 99` worked. A typed `EvaluatedPartSelectors` is now
what assignment-target parsers accept, obtainable only by exactly-once
evaluation or a stable-literal proof, so a future head cannot pass raw
selectors without failing to compile.

**Family B (briefs 379-383)** — seven defects, one invariant: the identity a
definition is stored under must come from the same attribute-aware preparation
as the identity a lookup constructs. Four failed to prepare, two prepared a
position that had to stay raw, and one needed the `Flat`/`Orderless`
normalization the ground index was not applying. Storage and removal now
travel as tokens whose constructors are private to one module.

**What the line is worth beyond the fixes.**

- *The bidirectional invariant was the thing.* Four defects failed to
  normalize and two over-normalized. Fixing only the common direction would
  have closed three and hidden the fourth — the shape the frame-roots line hit
  with `Block`'s bare `Symbol`. The user's decision to check WL behavior
  before ruling is what exposed the second direction.
- *A conclusion whose evidence is our own defect is not evidence.* plan-376
  recorded "raw rules become reachable after `Clear`" as resolved-with-evidence
  and reasoned from it. The behavior was real but was a by-product of the raw
  store itself; WL has no such effect. The whole preserve-raw option went with
  it.
- *Gate instruments must be calibrated before they are believed.* Every
  differential gate in this line was first shown to report a diff against a
  pre-family-A binary. The sealing test was calibrated the same way — one
  inserted call makes it fail with the exact count — and the compile-time
  boundary was checked by trying to forge a token (E0603/E0624).
- *Measurement lessons, three of them, each caught the hard way.* Best-of-2
  reports a spread four to five times too small: 1c left two labels open at
  +6.34%/+6.72% against 2.01%/2.96% spreads that are 10.6%/11.3% at
  best-of-3. Whether valid control labels exist depends on the change's
  reach — 1a sat on the common call path where nothing is untouched, 1b sat
  on the assignment path where 28 pure-computation labels decided the gate by
  moving *more* than the affected ones. And when a label looks like a
  regression but the disassembly is unchanged, a third binary carrying only an
  inert dead function settles it: in step 2 that arm moved the labels further
  than the real change did.
- *The same coincidence appeared twice, one step apart.* After 1b made `Set`
  store raw held forms, `Unset` on a held position "worked" because two
  unprepared expressions happened to be equal; after 1c, `TagUnset` did the
  same. Both had to be migrated in the step that would otherwise break them.

**Left open, deliberately.** Two guarded lookup labels moved +7.47% and +6.01%
in step 3, above the 0.4-4.1% spread measured between same-semantics binaries
differing only in layout, while the three hot lookup functions' disassembly is
instruction-identical with addresses shifted `+0xa0`. The layout estimate is
incomplete in a known direction — the inert arms add code where the change
removes 232 lines net. Recorded rather than tuned blind.

## 2026-08-31 — three long-standing items, then a consistency review (briefs 384-396)

`TODO.md` had carried three items for months. All three closed in a day, and
each turned on the same thing: a test asserting a property the implementation
never promised.

**The `rum-jupyter` ZeroMQ flake (brief-384)** had been seen three times since
brief-165, always green in isolation. It reproduces on demand in **release**
mode — 22 of 30 runs — and is nearly invisible in debug, which is what the test
gate runs. The test sent a shutdown on control, slept, sent an execute on
shell, and asserted the *next* event was control; the poller's priority is
correct but selects among channels already pending, and native transport
buffers on background workers. The repair waits until both channels report
pending through the same predicate the poller uses.

**Syntax error locations (brief-385)** were computed per form and rendered
against the whole file. A four-line script failing at 4:7 reported 2:1 — the
position of an unrelated character — and `Get` reported no location at all. The
suite could not see it: every pinned location was `1:x`, where form and file
coordinates coincide.

**Comparison chains (brief-386).** `TODO.md` recorded this as half-fixable
because `(a == b) == c` and `a == b == c` "share one AST". Both halves were
wrong — `a == (b == c)` flattened too, and the shared AST was a consequence of
parsing these operators as associative. The user's diagnosis was the right one:
they have no associativity. Chains are collected while parsing, and mixed
chains lower to `Inequality`, whose absence of a hold attribute is why each
operand evaluates once and nothing short-circuits.

**The call-stability flake (briefs 387-388)** turned out to be all three
recorded sightings of one cause: a 128-bit Bloom taking its bit from a
process-global interner id. E1 made it deterministic — 36 preinterned symbols
fail the direct test every time, 142 the indexed one — which was worth more
than the statistics that preceded it. The predicate is one-sided by design, so
the tests were at fault, and the repair pairs an identity-bearing target with
regular-lane dependencies so the lanes cannot intersect for any interner order.

**Then a four-part consistency review of briefs 364-388 (389-392), repaired by
393-396.** Eight findings, three high. Every repair replaced a convention with
a structure: the list of protected pattern wrappers became a registry shared
with the matcher; assertions resting on interner order moved to a
lane-disjoint construction; a public installer that trusted its caller now
demands a typed left-hand side; a wrapper claiming two constructors left the
continuation protocol that offered a third.

**What the review was worth, beyond the findings.**

- *Two findings were corrections to the briefs themselves.* The frame-root
  allowlist is an inventory of handwritten traversals, not a two-entry
  exemption list, and the frame queue is the single insertion route for
  ordinary code rather than literally. `CLAUDE.md` was wrong on both.
- *One was a gate miss.* brief-388 repaired the two assertions that had been
  seen failing; five more rested on the same mechanism, because the diagnosis
  and the repair both examined the recorded sightings, and the recorded
  sightings were two. **Sweep the class, not the instances.**
- *A boundary can hold inside the repository and evaporate at the crate edge.*
  Two findings were exactly this, and in both cases the test meant to police it
  scanned in-repository callers — **a source scan cannot establish a property
  about code that does not exist yet.** The gate that settled them compiled an
  external crate and required the compiler errors.
- *The worker's honesty was worth more than several of the findings.* It
  reported that native tests skipped in its sandbox and that its green rows
  were compilation evidence rather than stability evidence; that two assertions
  could not be made to fail and so were reported rather than changed on
  suspicion; and that a brief's own background was wrong. Where its evidence
  could not exist, the verification became the director's: forty release
  repetitions on a host where those tests actually run.

## 2026-09-02 — the suspension and cancellation line (briefs 397-411)

The project moved from macOS to Linux, and the first release-profile full-test
run on the new host produced two `rum-jupyter` failures the debug gate had
never shown. Fifteen briefs later, three kernel defects and a class of invalid
test assumptions are closed. Endstate: workspace 2287/0/7, the native ZeroMQ
suite 0 failures in 30 repetitions on a host where it demonstrably runs.

**What was actually wrong.**

- *Cancellation was erased by the evaluation it was meant to stop.* The
  prologue cleared a session-global flag as `eval_request`'s first statement,
  and the jupyter adapter takes its handle before spawning — so the handle was
  obtained at the right moment, the cancel was set at the right moment, and the
  evaluation threw it away. Now request-scoped and generation-qualified.
- *Abandoning a suspension leaked every scope its frames owned* — `Block`,
  `Module` activations and gensym reclamation, iterator scopes, `OptionValue`,
  `Reap`, `Monitor`, constraint deadlines, trace and capture checkpoints,
  `AbortProtect` depth, `BlockRandom`, partial package installs. The leaked
  `AbortProtect` depth let evaluation continue past `Abort[]`. Reachable
  through public `rum-host` API alone.
- *Overlapping public requests silently replaced a pending continuation.* Now
  rejected, by user ruling; supersede stays unimplemented rather than being an
  accident.
- *The jupyter test suite asserted timing properties the implementation does
  not promise*, in three unrelated families plus an ephemeral-port TOCTOU.

**The lessons worth keeping.**

- *An instrument reported nine skips as nine passes.* A native ZeroMQ test that
  cannot bind loopback printed one line to stderr and early-returned, and
  libtest said `ok`. This had already cost the project a false 0-of-30
  stability result in brief-384. It is now a red test unless a named
  environment variable opts in. **Silence had been an option; it no longer is.**
- *A worker's green gate is worth exactly what it can execute.* brief-409
  passed its own gate and broke all nine native tests on the director's host,
  because the panic it introduced was masked by the same skip path. The repair
  was one line; the durable fix was socket-free coverage of the harness, which
  paid off two briefs later when family B needed a gate the worker could run.
- *Refuting a repair is worth more than performing one.* brief-401 killed
  brief-400's proposed mechanism — driving saved frames with `Err(Cancelled)`
  is not an unwind, because ordinary frames consume errors and run work.
  Splitting that defect out was a hypothesis at the time; had it been wrong,
  the repair would have gone back inside brief-400.
- *Measure the mechanism, not the outcome.* A probe that varied only what
  happens between dispatch and the first completion poll showed cold, warm,
  worker-warm and yield all reading not-ready and only a sleep reading ready.
  That is what forbade a spin as the fix, before anyone wrote one.
- *After families A and D there was nothing failing*, and C and B still had to
  be done. A test that passes because a machine happened to be fast is
  supplying false confidence, so "it still passes" stopped being evidence and
  the gates became structural.
- *Waiting cannot prove absence.* Two tests waited 200 ms, saw no reply, and
  concluded none was produced. They now read state that says so.
- *My own probes lied three times in one session* — a `git stash show`
  invocation that reported a conflict for every file, a `grep` for
  `test result: ok` that is always true when several test binaries run, and a
  bare test name passed to `--exact` that matched nothing and was counted as
  failure. Same shape each time: trusting a predicate without first checking it
  against an answer already known. The verification harness needs calibrating
  as much as the code does.

## 2026-09-03 — the memory-accounting and traversal-depth lines (briefs 412-427)

Opened by paying an owed verification and closed nothing; two lines are part
built and both are recorded in `TODO.md` rather than here.

**The first security audit this project has had** (brief-413, GLM, no scope
narrowing, because Codex's cybersecurity filter makes such work impossible
there). Seven findings, two HIGH, both the same root: bytes the memory model
cannot see. The process-global symbol interner leaks by construction and is
invisible to `eval_memory_limit`; the ceiling is sampled every 64 evaluation
points and can be overshot multiplicatively in between. Its three criticisms of
the threat model were worth more than several of the findings, particularly
that no document says which state is per-request and which is per-session.

**The memory-accounting line (415-420).** Reviewing the design turned up a
third gap the audit missed: `current_memory_proxy_bytes` walked 34 of
`Session`'s 93 fields, so the baseline every ceiling decision measures against
was itself short. A macro now generates both the checked-in classification and
an exhaustive `let Session { .. }` pattern, so a new field stops compiling
until it is classified — verified by adding one and watching the compiler
refuse, after a first probe that only tripped the constructor and proved
nothing.

Then `MemoryGrowthContract` became a required argument to all nine
`BuiltinSpec` constructors across 709 construction sites, and — the part that
mattered — a falsifier that fails a declaration the handler contradicts. It
caught eleven wrong declarations immediately, `Rule` returning 64,289 bytes
from 424. brief-418 then found the falsifier armed on a production memory
ceiling, where it cost 2.2x and could panic, and unarmed in every crate but
`rum-runtime`, where `cfg!(test)` does not reach. brief-419's `Preflight`
falsifier named eighteen more; ten were repaired and eight retained with argued
reasons. **Five of the eleven that brief-417 reclassified had landed in
`Preflight` while it had no teeth** — a deferral wearing a repair's clothes,
which neither the worker nor I noticed at that gate.

**The traversal-depth line (422-427)** began with a claim of mine that Codex
refuted: I wrote that all eight retained decoders had depth guards after
checking three of four. `FromXml` had none, and 12,000 levels of nesting built
with two `Pure` string builtins ended the process outside the panic boundary.
`TODO.md` records why: the previous repair bounded "all three codecs" when
there were four.

Four descents were bounded and ten traversals converted; the bound itself is
not installed, because caching the height that makes admission O(1) costs 30%
on list rebuilds — `SharedArgs::new` went from `Arc::new` to scanning every
argument. The stop-loss fired and the spike is in the stash.

**Method, and it is mostly about instruments.**

- *An instrument that has never been seen to fail is not known to work.* Both
  falsifiers ship with checked-in `#[should_panic]` proofs, one through eager
  dispatch and one through suspend/resume. I verified each myself by
  misdeclaring a real catalog builtin and watching the suite go red.
- *Seven probes of mine measured something other than what they claimed*, all
  the same shape: `git stash show -p` with a path, a `grep -q` that is always
  true, `Nest` stopping at the iteration limit before building anything deep,
  `Level[x, {2}]` barely descending, `grep -rl | head -1` returning a file that
  references a constant rather than defining it, summing two inventories
  without checking their overlap, and `MatchQ[x, _]` taking a fast path that
  never reaches the code under test. The discipline that catches these is
  calibrating against a known answer before believing a result.
- *Codex's content filter is not a wording problem.* It killed brief-422 twice
  — the second time after the security framing was gone, so the trigger was the
  reproduction transcript — and brief-424 twice more, at 120k tokens, on the
  worker's own generated probe code. The judgement is whether a brief needs an
  executable reproduction, not which words it uses. brief-424 I ran myself;
  brief-427 was rewritten as source reading, which is what a "why" question
  wanted anyway.
- *A report that exists only in a dispatch log is one reboot from gone.*
  brief-414's inventory was never appended, and brief-417 stopped without
  touching a file because the authority it was told to use was not in the
  repository. Appending the report is now the first thing I do with a returned
  brief.

## 2026-09-03 (later) — the candidate normalizer's recursion (briefs 428-433)

The 4,500-height ceiling six converted traversals shared turned out not to be
in the traversals. A single `MatchQ[x, _?IntegerQ]` — no traversal at all — died
in the same bracket as an all-level `Count` over the same value, while
`MatchQ[x, _]` cleared 20,001 through the blank fast path. The whole ceiling was
one recursive normalization of the match candidate, which is now bounded:
43,001 for the single match, 20,001 for the traversals, against a ruled minimum
of 4,097.

**Six briefs, three of them the same repair in three shapes.** The design
(428) was sound and the plumbing was not. Propagating exhaustion as a `Result`
cost 6-15% on wide traversal normalization, because the depth budget is
consumed only on call nodes but the return type was paid on every atom.
Handling exhaustion locally removed that and left ~8% on `matcher-single-*`,
because atoms still entered a function carrying a depth parameter their arm
ignored. Giving the recursive worker a decomposed call — so an atom cannot be
passed to it — took it to parity. Each shape was predicted before it was
measured, and each prediction named what would falsify it.

**The measurement lessons are the durable part of this line.**

- *A single base/candidate pair cannot gate this panel.* brief-431's stop-loss
  named two coherent regressing families. Four passes in one window put the
  same-binary spread at −7.78% to +11.19%: `matcher-single-wide-small` moved
  +11.19% between two runs of the identical binary, larger than the regression
  attributed to it, and its two replicates read +13.31% and +0.11%. Half that
  stop-loss was noise, and the protocol that produced it was the director's.
- *Layout is not noise and has to be sampled separately.* A third binary — HEAD
  with one function moved to the end of its `impl` block, identical semantics —
  moves this panel at most 4.45% and the `matcher-single` family 0.10%-1.52%.
  Without that number there is no way to say whether an 8% movement is the
  change. The first attempt at that control used `#[no_mangle]`, which does not
  compile here without an `unsafe` attribute the project forbids; the `cp` that
  followed silently copied the previous binary, and only the hash check caught
  that the "control" was the candidate.
- *When every affected label moves the same direction by a similar amount after
  control adjustment, the adjustment is the artifact.* The final A/B read +1.3%
  to +5.3% across the whole panel — including labels the change cannot touch.
  The within-run quotient against a same-tier untouched neighbour, which is
  immune to a whole-run offset by construction, read −0.52% to +2.64% with
  every label inside its own same-binary quotient spread. That is the measure
  that settled it.
- *A liveness probe can be the thing that fails.* `MatchQ` at height 43,000
  failed with a `Depth[x]` prelude and passed without it: `Depth` is its own
  unbounded recursion and gives out between 20,001 and 43,001. The checked-in
  certificate therefore excludes `Depth`, and a separate test carries the
  height assertion.
- *Three attempts is a plan, not drift, when each one is falsifiable.* Each
  brief named the labels that would have to move and the number at which to
  stop and hand the decision back, so a fourth shape was never on the table.

**Method that worked and should be reused.** The repair was split into three
gated steps — lock the contract in tests, move the module, change the
algorithm — so the semantic diff was readable on its own. The contract lock is
a frozen transcription of the old algorithm compared against production on
three axes per case: the `Option` discriminant, the logical value, and which
input `Arc`s the output reuses, mapped over the whole output tree. That third
axis is the one that matters, because pointer identity here is an observable
contract: the evaluation-stability cache validates entries by exact pointer
equality, and its uncached path is another recursion. Both red calibrations
were recorded before the green run, and the director reproduced the sharing one
independently.

**A record correction.** `7ae6aa3a` is labelled "docs" but also carried
brief-432's implementation of three source files; a `git add -A` swept it in.
History is left unrewritten and the fact is recorded in `0f679699`.

## 2026-09-04 — the native stack guard that could never fire (briefs 436-438)

Opened as the first of four repairs for traversals short of the ruled 4,097
height, and the first one turned out not to be about `StringExpression`.

brief-436 (diagnostic) found the mechanism: a strict head whose call stays
residual and never acquires an evaluation-stability proof is re-evaluated
through eager argument preparation on every construction step, one native
evaluator level per spine level. All three of its offered predictions held at
the gate, and it correctly refuted my `Sin` control — `Context[Sin]` is
`Global`, so that probe was a second copy of the `foo` control. Its recommended
repair reached half the class; `Power`, `Mod`, `Max`, `Min`, `Equal`, `Less`
and `Not` carry real dispatch and are in the class too.

The gate found what the diagnosis stopped one step short of.
`poll_native_stack_limit` measures bytes actually consumed, polled every four
levels — stack-honest by construction — but its default was 6 MiB against the
ruled 2 MiB embedder stack, so on the ruled configuration it could never fire.
The same construction returned `reclim` in band on 8 MiB and took SIGABRT on
2 MiB. `SECURITY.md:43-60` published "Exceeding one yields an in-band
`TerminatedEvaluation[...]` value, not a crash", which was false there, and
`codecs_xml.rs` already carried the workaround comment.

brief-437 (design-only) measured the frame: a 4,736 B static packet over seven
resident frames, against my 4,675 B bisected slope — 1.3% apart by two
independent methods. It corrected my inventory (`run_call_dispatch_stage` runs
on ascent, not during descent) and projected a 1.5-2.6 KiB optimization floor,
which settles that height 4,097 for a strict residual call needs an explicit
task stack rather than frame polishing.

Its ripple inventory was priced on an unqualified "approximately 20 KiB"
quoted from the 2026-07-06 entry, without noting that it is a **debug** number
while its own packet came from a release binary. I measured all three — 4.7
KiB release strict, 7.5 KiB release user recursion, 20.6 KiB debug — which
reversed my own first reading: I had taken 7.5 KiB and concluded the projected
test breakage would not happen, when `cargo test` runs debug on a 2 MiB thread
where the depth-72 guard-adjacent test already consumes 1.48 MiB. Two invalid
probes were caught on the way: a literal nested source pinned at 125 levels at
every stack size because it was hitting the parser's 128 nesting guard, and a
region above 2 MiB pinned by the fixed-point pass limit rather than the stack.

brief-438 landed it. Four tests broke; all four were repaired by declaring a
thread and deriving the budget from the same size, with depths and `reclim`
assertions preserved — I rejected the draft's own proposal to take the
guard-adjacent tests from 96/72/120 down to 32/24/40, which would shrink
coverage to fit the budget. The `codecs_xml` workaround is retired and its
acceptance coverage restored from 64 to the full 256.

Gate: `cargo fmt --check` clean; workspace 2329/0/19 **without** the ZeroMQ
skip flag the worker's sandbox needed, so the nine native tests it could not
run actually ran here; 12 release depth certificates green on 2 MiB under the
tighter budget, confirming the 422-433 traversal work holds; both rewired
ignored tests green. Behaviour verified directly: `Power`, `StringExpression`
and `General` at N=3000 on a 2 MiB stack went from exit 134 to exit 1 with
`General::reslim: evaluation exceeded native stack limit (1048576 bytes)`.

Three items stay open: the strict-residual class is still quadratic in
construction (brief-436's options A and B), the explicit task stack that would
actually reach 4,097 is unscheduled, and the byte guard still cannot see a
recursion that runs entirely inside one builtin.

## 2026-09-04 (later) — the deep-rebuild class and the stability walk (briefs 439-446)

Opened as "dispatch the remaining three traversal repairs" and re-probing them
under the new 1 MiB budget found one shared defect instead of three.

**brief-439 (design-only).** A traversal that rebuilds a deep result hands the
evaluator a tree with fresh `Arc` identities at every level, so every stability
cache key misses and `call_stability_uncached_with_bloom` walks the whole tree
through `cached_expr_stability_with_bloom` — mutual recursion in a file where
`evaluation_depth` never appears, so the byte guard has no poll point for the
entire descent. That is why `ReplaceAll` on a match, `Apply` at a descending
spec and `Replace` at all levels died the same way, and why the same operations
with a rule that never matches survived: the traversal returns
`Expr::call_shared` with the original pointers and the root's proof still hits.
brief-425's conversions were not wrong; the shared post-result check downstream
of them was never converted.

**The certificate panel had a matching hole, and it is the session's largest
finding.** None of the twelve exercised a rebuild. `Map` and `Apply` used
`{-4097, 0}`, which in rum selects level 0 only — intended, test-pinned
semantics, verified against `traversal.rs:142-149` — so they wrapped the value
once and handed it to an already-iterative `Depth`. `ReplaceAll` replaced a
symbol absent from the value. All three now assert the exact rebuilt
`InputForm` string, and each is paired with a shallow companion that pins its
selector on a three-level fixture. Those companions are the falsifier: pointed
at the old selectors they fail, and they need no deep value, which matters
because the pre-fix deep case aborts the test binary.

**Two repairs were rejected before the third worked.** briefs 440 and 441 made
the walk iterative behind a depth-bounded front. Both were functionally
complete — every invariant covered by differential tests including an exact
cache-insertion-order trace, twelve certificates green, counters identical —
and both cost. Reproduced in my own four-pass window with within-run quotients:
`downvalue-exact-miss-repeat` +5.6%/+6.9% against a 0.62% base spread,
`cases-wide` +5.7%/+3.8%, `matcher-single-tiny` +2.4%/+2.2%. A depth parameter
and an `Option` return on the most-consulted function in the system is not
free. Preserved as stashes.

**What worked was moving locals, not control flow (brief-442).** The walk cost
811 B per level and its dominant frame was 584 B — not because the function is
large but because the whole chain is inlined into it, making the frame the union
of thirteen data-constructor arms, cache lookup and insertion, head-effect
computation and profiling tokens. A deep descent executes almost none of that.
Outlining them gave 255.6 B/level measured, a 3.17x reduction, with the
previously-regressed families neutral to faster.

**brief-443 measured three fail-closed shapes and none fit** — a per-level floor
check grew the frame to 272 B and bailed at 3,797; the same check on a stride
held 256 B but bailed at 4,040; a bare depth counter kept the certificates but
was not stack-size-honest. All three would have turned nine certificates into
in-band failures. A well-measured no was the outcome, and it found the route
out: about 64 bytes of off-by-default profiling state resident in the hot
recursive frame, which is a hard-line violation independent of any poll. Its
method is worth keeping — gdb ptrace is denied in the worker's sandbox, so it
measured the slope from `/proc` VmStk high-water at four heights, without a
single abort.

**brief-444 removed that state.** The key is a pure function of the head, so it
is derived inside the cold enabled path rather than carried; the start instant
moved to a thread-local stack. Prologue 248 → 184, walk 256 → 192 B/level.
Height 9,500 completes on 2 MiB where 8,008 was the ceiling. The per-head
profiling surface had no test anywhere; it has an env-gated one now, run against
HEAD before the change and pasted.

**briefs 445-446 made the walk fail closed.** The strided poll fits at 192
B/level: bail-out bisected at my gate to 5,339 ok / 5,341, a margin of 1,242
levels (30%) over the ruled 4,097, and height 20,001 on a 2 MiB stack moves from
exit 134 to an in-band `General::reslim`. The bail-out is the existing
early-unstable return of `(false, bloom)` — stability only decides whether
evaluation may be skipped, so a conservative `false` is always safe and needs no
error channel. brief-445 put the counter at the walk's entry, where it also ran
for atoms and cache hits and cost `list-literal-small` +7.99%/+7.98%; moving it
to the call-cache miss path removed that entirely and is the more honest meter.
Two codec depth tests surfaced with the budget enforced and were given declared
threads like their XML sibling, not a lowered depth.

Gate at each step: workspace 2333/0/20 without the ZeroMQ skip flag, twelve
release certificates, the profiling certificate, and a four-pass panel I ran
myself rather than reading. My own instrument lied twice — a parser reading the
wrong filename that printed a table of zeroes, and a bisection whose binaries
this host had wiped from `/tmp`, so every run failed with exit 127 and it
converged on the low bound. The calibration point caught the second.

Still open: brief-439's steps 2-8, the per-site conversions whose projections
rest on brackets measured under the old 6 MiB budget; the same profiling-state
defect in the evaluator's own 4.7 KiB/level chain, worth an order of magnitude
more than the one just fixed; and three files over the 35k organization line.

## 2026-09-05 — the deep-rebuild tail, and three instruments that lied (briefs 447-456)

Ten briefs closing brief-439's tail and the association class brief-454 opened.
Every one of them was scoped by re-probing first, and **every re-probe moved the
scope** — that is the finding of the session, not any single conversion.

**What landed.** `Replace` (1,308 -> 5,843, fail-closed), `D` (1,738 -> completes
at 4,097), `DeleteCases` (1,808 -> 5,408), multi-spec `Take` (2,573) and
deep-path `ReplacePart` (2,854) both to 4,097, the lexical and rule-binding
substitution engines (3,385 and 4,430 -> 5,867/5,843), the association arm of the
stability walk, and `MergeNested`. The last three association sites — `Map` over
a deep association, a read after nested key-path assignment, and `MergeNested` —
now run to the construction ceiling at 5,457 wrappers, so the operation is no
longer the limit in any of them.

**Re-probing changed the plan five times.** draft 5 lost six of its nine named
consumers, which already cleared the contract; draft 6 split into two briefs
because `DeleteCases` and the substitution engines had different shortfalls and
different caller surfaces; slot substitution left the scope entirely once a
probe using a real `Slot[1]` replaced one that used a named parameter and
therefore measured the lexical engine; rule-binding substitution entered the
scope despite clearing the contract, because its 1.08x margin was thinner than
the ~8% these brackets drift between unrelated builds; and `MergeNested` flipped
from fail-closed to fail-open the moment brief-455 removed the polled cost that
used to be hit first. **Removing a guard-adjacent cost can convert a
fail-closed site into a fail-open one.**

**The measurement lane was rebuilt mid-session, on a user ruling.** This host has
P-cores and E-cores 1.27x apart in clock with no pinning, so a wall-time
percentage is an indication and never proof. Two worker stop-losses (+0.61% and
+7.14%) turned out to be noise; the second needed a layout-control binary to
settle. `CLAUDE.md`'s standing "counters beat timings" rule had lapsed and is
reinstated, and the lane now has three instruments: allocation counts
(`RUM_PROFILE_ALLOC`), retired instructions (`perf stat -e instructions:u`,
readable unprivileged here), and depth brackets, which are integers and immune
to all of it.

**Three instruments lied, and each cost a wrong conclusion before it was
caught.**

- *The hybrid PMU.* A run that migrates between `cpu_atom` and `cpu_core` reads
  about 55% high. Discard it; both counters non-zero is the tell.
- *The build path.* Comparing a binary built in a `git worktree` under `/tmp`
  against one built in the repository read **-8.4%** on probes the change could
  not touch. Same-shaped paths bring it to **+0.005%**. At the brief-455 gate I
  attributed those numbers to codegen from a module growing 444 KB and shipped
  that attribution into the record; the brief-456 gate refuted it. The tell was
  already in my data — two unrelated changes produced the same figure on the
  same untouched probes — and I stopped at the first explanation that fit.
- *The height unit.* One logical association wrapper is two physical levels,
  because `Association["k" -> v]` renders as `Association[Rule[…]]`. I compared
  wrapper counts against a physical-height contract and overstated three
  shortfalls twofold. `Depth` hides the `Rule` layer and must never be the unit.

**A worker diagnosis was accepted and one of its sections refuted.** brief-454
correctly identified that `Map` and post-assignment reads are the association
arm of the same stability walk this line had already shrunk on its generic arm,
carrying a 400 B association helper and a 112 B rule helper above the 192 B
generic packet. Its attribution of the shared 5,455 bracket to `Expr` teardown
was wrong: construction alone brackets identically, in band, before any output.
That mattered because the teardown reading would have sent a later reader to
optimize `Expr::drop`, a lane the standing debt already records as tried and
rejected.

**One change was accepted over its own stated bar, deliberately.** The lexical
substitution front costs +0.714% instructions on `With[{x = 1}, x + 1]` against
a ±0.15% bar. Two shapes were built and measured; the alternative, a
thread-local depth, was slower at +1.085% and carried five manual enter/leave
pairs with no `Drop` guard, whose breakage would silently shrink a thread's
recursion budget. The cost buys the removal of a process kill at 3,385 levels,
and the decision is recorded as a measured trade rather than a waiver.

**Dispatch failures, cause open.** Three dispatches were terminated mid-run with
no error text and no exit line, each after edits had landed and before
verification. It is not the content filter — the three recorded filter kills
ended with `exit 1` and a policy message — and not a size or duration ceiling,
since the three failures are the three shortest logs in the line while a 15.3 MB
run over 18 minutes completed. Recorded as sightings; the recovery that worked
was to shrink the brief and keep the long verification on the director's side.

Still open: brief-454's drafts C through J, `MapIndexed` at 2,694 (fail-closed),
and the second recursion inside `Take`/`ReplacePart` that terminates the process
above roughly 11,000, for which `merge_replace_plans` is the named candidate.

## 2026-09-05 — the association depth class re-derived; six of eight drafts dropped

brief-457 (design-only) re-derived brief-454's drafts C-J against `80c8acce`,
and the director bracketed every site in parallel. The split was deliberate: a
past-ceiling probe is not delegable in this line — brief-454 forbade the worker
from running one, and the three recorded content-filter kills were all
executable failure reproductions — so Codex got the source-side derivation and
the brackets stayed here.

**The re-pricing was the point.** brief-455 was the shared prerequisite every
one of drafts C-J was priced against; removing it re-priced the class. Six of
the eight drafts are dropped because their priced target now clears the ruled
4,097, the same outcome brief-450 reached when six of nine consumers already
cleared. What survives is one draft (multi-spec `GroupBy`, 2.03x short and
fail-open), one lane brief-454 had deferred (`Keys` and `KeyTake` over nested
lists, both short and fail-open), the generic-shape `MapIndexed` post-result
ceiling, and the schema validator.

**Four things from it are method, not history.**

- *A ceiling is not a deterministic quantity, and this line has been reporting
  it as one.* Inside a band a few tens of levels wide the same height flips run
  to run — `DeleteMissing` read 3 PASS / 2 KILL over five runs at one height —
  because the stack base moves between processes. A single-run bisection reports
  a bracket 8 wide when the real uncertainty is nearer 50. Every bracket in this
  entry is the highest height passing 3/3 against the lowest failing 3/3, with
  the failure direction read well inside the stable-fail region.
- *Reasoned packet arithmetic is reliable for an operation-local native walk and
  unreliable the moment the descent crosses the evaluator.* Ten of twelve
  projections landed within 2-25%, four within 2%. Both failures are the same
  error: an evaluator crossing priced as a native frame. The schema validator was
  reasoned at 1,986 pairs failing open and measures 104 pairs failing closed —
  19x, and the direction wrong — because it crosses the evaluator at every
  nested pair, not only at the leaf. `Keys` was reasoned to clear and does not.
- *Removing a guard-adjacent cost converts fail-closed into fail-open, and it
  did so to an entire class.* brief-456 saw this in one operation. Measured
  across the class, **every** site that now clears the contract terminates the
  process above it and not one fails in band: brief-455 lifted them over 4,097
  and removed the polling frame they had been borrowing in the same stroke. The
  contract is met with 1.5x-2x margin; the fail-closed promise is not. That is
  now the largest open item here, ahead of any individual site.
- *Decompose a composite probe before believing its attribution.* The schema
  probe varies two axes at once. Split, a deep value against a shallow schema
  clears outright and only a nested schema descends — so the site's axis is
  schema depth, and a repair aimed at value traversal would have missed it.

**The instrument was calibrated before anything was believed, and it lied
twice.** The classifier was shown printing each of its four verdicts on a known
case; its first known process-kill case was wrong, because `ToString[_,
InputForm]` runs on the iterative `RenderTask` path and survives 30,000 levels,
so the kill branch had never been demonstrated until `FullForm` replaced it.
Nineteen probes were then calibrated at height 3 against exact deepest-target
assertions, which caught three broken ones: a `Scan` count off by a factor, a
malformed `Merge` collection, and a `KeyDropFrom` written with bare string
selectors — that last one returns `invak`, which reads exactly like a depth
failure and is not one. A literal deep association source turned out to be
unreachable entirely: rum refuses source nesting past 128.

**A stated worker risk was settled by measurement rather than argued.**
brief-457 named `GroupBy`'s failure direction as its largest likely
contradiction, on the reasoning that a callback with enough evaluator depth
might reach a poll and fail in band. It does not: the direction stays a process
kill and the ceiling drops below 1,008 specs.

Still open: the class-wide fail-open direction above the contract, multi-spec
`GroupBy`, the deferred collection-shape lane, generic-shape `MapIndexed`, the
schema validator's per-pair evaluator crossing, and — unchanged from the last
entry — the second recursion inside `Take`/`ReplacePart` above roughly 11,000.
