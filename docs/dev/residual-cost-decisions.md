# Residual cost decisions — the class-form publication and the evaluator fastpath

Status: decided 2026-11-26 (decision gate c205, ruling 857). The user adopted the director's
recommendation. This record is written to be revisited: it states the question, the evidence,
the decision, the rejected alternatives and the conditions that should reopen it. Repo docs are
English by convention; the working record for the same decision is `.temp/director/ruling-857.md`.

## Where the question came from

The D.1/D.2 residual repair line (briefs 798–809) measured the armed-vs-real whole-panel
residual, dispositioned every family of the 1,743-row ledger (93 families), and closed with no
landing (gate c204, ruling 856). Two items were not measurement questions and were reserved to
the user:

1. the class form's publication mechanism costs **1,464,035–1,464,239 Ir** on every whole-panel
   run, invariant across armed and real; and
2. the evaluator **fastpath** placement saves ≈44.6M Ir in both roles but its armed-minus-real
   differential, **41,712 Ir**, is inside its same-code band, **51,790**.

## Decision 1 — publication: funded (kept)

**The evidence.** The class form publishes through 38 gates: 37 independent at 4 Ir/visit and
1 reused at 3 Ir/visit. The baseline is 363,371 visits × 4 + 3,517 × 3 = 1,464,035 Ir; the
draw-dependent Set gate raises the top to 366,939 visits and 1,464,239 Ir. The fresh epoch
measured 1,464,035 / 1,464,147 / 1,464,199 across its three armed draws (brief 807f), and the
809 publication verification reads the 37 invariant source visit counts plus the draw-dependent
Set gate in all 13 cells (slow self 0). The amount is identical in armed and real: it is the
publication mechanism's price, not a residual of the A/B.

**The decision.** Book 1,464,035–1,464,239 Ir as the class form's necessary cost. Do not open a
redesign line. The class form proceeds into the epoch with this cost accepted and accounted.

**Why.** The publication mechanism has not been falsified: every invariant it promises (owners,
order, visits, the 37×4 + 1×3 shape, 4 Ir per visit) is measured and verified. A redesign would
reopen the class form's structure and re-prove those invariants with no evidence of a better
mechanism; the standing rule "pick the best and most correct option" does not favor reopening an
unfalsified design, and the cost's size is not an argument either way.

**Rejected alternatives.** Redesign now: no falsified mechanism, no concrete design, and it
would reopen the class form and the campaign's epoch. Hold: an unbooked structural cost leaves
the epoch's account incomplete.

**Revisit if.** The class form's publication mechanism is reworked or re-scoped; the 38-gate
invariant (owners, order, visits, 4 Ir/visit) changes; the measured cost moves outside
[1,464,035, 1,464,239] beyond the draw-dependent band; or a concrete redesign is proposed with
evidence of a strictly better mechanism.

## Decision 2 — fastpath: excluded (per 804)

**The evidence.** The fastpath union is −44,623,356 armed and −44,581,644 real (a saving in
both roles); the armed-minus-real differential is 41,712 Ir. The same-code bands are 20,120
(armed, 14 draws) + 31,670 (real, 3 draws) = 51,790, so the differential is numerically
unseparable. The placement saves ≈44.6M in both roles: a default-path optimization, not a
residual repair.

**The decision.** Exclude it from the repair package and from this line. Do not land it; book no
credit. The default path stays as it is. A default-path optimization line remains a separate
future decision.

**Why.** The only residual-repair currency (the 41,712 differential) is inside its own band, so
it cannot be booked as a repair. As an optimization it is outside this line's scope and would
need its own single-variable A/B and default-path cost gate; opening that line now would widen
the scope without a user decision to do so. Brief 804 had already excluded it as a general
optimization, and brief 805's landed package is the seven outline attributes plus the coldout
refactor at the definitive joint price 13,556,424.

**Rejected alternatives.** Land now: out of the residual-repair scope, unseparable in its
residual currency, and it would need its own measurement line. Hold: the item is closed as
excluded; reopening requires the conditions below.

**Revisit if.** A default-path optimization line is authorized; further same-code draws tighten
the band below 41,712 or the differential grows beyond it; the fastpath placement or the
evaluator's flag-address mechanism changes; or the campaign's scope comes to include
default-path optimizations.

## Accounting effect

- Publication booked; fastpath not credited. The D.1/D.2 residual account has no open user
  items.
- The class campaign's Stage C/C10 readmission and the 744 restart are unblocked by these
  decisions; they remain subject to the campaign epoch being built and qualified under the v2.5
  interference budget.
- 810 stays cancelled; the residual repair line stays closed; the 798–809 records stay sealed.
- Budget: no charge for the decisions; pool 6,041.152 s; cumulative 22,358.848 of 28,400 s.

## Pointers

- Gate c204 / ruling 856 (the measurement and the returned items); decision c205 / ruling 857
  (this decision); brief 804 (the D.1 decision table); briefs 802/803 (the fastpath cells);
  brief 807f (the fresh epoch's three draws); brief 809 (the joint measurement and the
  publication verification).
- Sealed records: `.temp/brief-804/`, `.temp/brief-807f/`, `.temp/brief-809/`.
- Working state: `.temp/director/state.md` continuation 209.
