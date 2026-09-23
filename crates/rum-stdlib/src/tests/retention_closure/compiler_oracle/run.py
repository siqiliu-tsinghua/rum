"""Compile real-source mutations and challenge the instrument itself.

This checks Rust facts with rum's stable compiler, not rum behavior against a
third-party WL engine. No compiler/RA implementation code is copied.

The extra frozen carrier in type_probe is motivated by rustc's argument
coercion path in rustc_hir_typeck/fn_ctxt/checks.rs and coercion.rs:
https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_hir_typeck/fn_ctxt/checks.rs.html
https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_hir_typeck/coercion.rs.html
Reading compiler source does not use nightly: every build pins stable 1.85.0.
An explicit assert_type::<T>(expr) ALSO permits coercion. A separate statement
infers a nominal invariant carrier before the expected type is introduced.
"""

import csv
from pathlib import Path
import shutil
import sys

sys.dont_write_bytecode = True
from scratch import Scratch, read_tsv


def agreement(positive, negative):
    assert negative is not None, "negative control disabled: resolution evidence is incomplete"
    return positive and not negative


def type_probe(expression, expected, mode="exact"):
    setup = "fn assert_type<T>(_: T) {} "
    setup += "struct Frozen<T>(T, std::marker::PhantomData<fn(T) -> T>); "
    setup += f"let value = {expression}; "
    if mode == "direct-coercion":
        check = f"let borrowed = &[value]; assert_type::<&[{expected}]>(borrowed); "
    elif mode == "frozen-coercion":
        check = "let frozen = Frozen(&[value], std::marker::PhantomData); "
        check += f"assert_type::<Frozen<&[{expected}]>>(frozen); "
    else:
        check = "let frozen = Frozen(value, std::marker::PhantomData); "
        check += f"assert_type::<Frozen<{expected}>>(frozen); assert_type::<{expected}>(value); "
    return "{ " + setup + check + "value }"


def run(workspace, directory, features):
    # A failed run must not leave an earlier run's successful control artifacts
    # looking current. Keep only build caches, the source copy and input claims.
    for path in directory.iterdir():
        if path.is_file() and path.name not in ("type-claims.tsv", "resolution-claims.tsv"):
            path.unlink()
    if (directory / "mutations").exists():
        shutil.rmtree(directory / "mutations")
    scratch = Scratch(workspace, directory, features)
    results = []
    controls = []
    try:
        status, diagnostics = scratch.compile("baseline")
        assert status == 0 and not diagnostics, "unmutated scratch must compile"
        print("ORACLE_BASELINE stable=1.85.0 compile=pass", flush=True)
        for row in read_tsv(directory / "type-claims.tsv"):
            name, exact = row["name"], row["exact_type"]
            observations = {}
            for mode, expected in [("exact", exact), ("wrong", "usize" if exact == "bool" else "bool")]:
                label = f"type-{name}-{mode}"
                scratch.expression(label, row, lambda expression: type_probe(expression, expected))
                status, errors = scratch.compile(label)
                observations[mode] = (status, bool(scratch.at(errors, row["file"], row["start_line"], "E0308")))
                scratch.restore()
            assert observations["wrong"][0] != 0 and observations["wrong"][1], "wrong type control did not fail at its expression"
            agrees = observations["exact"][0] == 0
            results.append(("type", name, "AGREEMENT" if agrees else "DISAGREEMENT", row["file"], row["start_line"], row["target"], exact))
            print("ORACLE_TYPE", name, results[-1][2], f"site={row['file']}:{row['start_line']} exact={exact} wrong_type_caught=true", flush=True)

        # Watch the brief's direct generic form accept a coercible wrong claim,
        # then watch the frozen-unification guard reject that same wrong claim.
        row = read_tsv(directory / "type-claims.tsv")[0]
        for mode in ["direct-coercion", "frozen-coercion"]:
            label = "control-" + mode
            scratch.expression(label, row, lambda expression: type_probe(expression, row["exact_type"], mode))
            status, errors = scratch.compile(label)
            scratch.restore()
            if mode == "direct-coercion":
                assert status == 0, "direct generic coercion premise changed; review the control"
                print("ORACLE_FINDING assert_type::<&[bool]> accepts &[bool;1]; direct generic call is not exact unification", flush=True)
            else:
                assert status != 0 and scratch.at(errors, row["file"], row["start_line"], "E0308"), "frozen type control failed to reject coercion"
        controls.append(("type", "pass"))

        caught_wrong_claim = False
        negative_disabled_caught = False
        for row in read_tsv(directory / "resolution-claims.tsv"):
            name = row["name"]
            positive = negative = None
            for which in ["target", "negative"]:
                label = f"resolution-{name}-{which}"
                scratch.signature(label, row, which)
                status, errors = scratch.compile(label)
                scratch.restore()
                assert status != 0, "signature mutation was not exercised by a compiled call"
                if which == "target":
                    positive = bool(scratch.at(errors, row["file"], row["start_line"], "E0061"))
                else:
                    negative = bool(scratch.at(errors, row["file"], row["start_line"]))
                    assert scratch.at(errors, row["control_file"], row["control_start_line"], "E0061"), "negative mutation was not independently exercised"
            agrees = agreement(positive, negative)
            results.append(("resolution", name, "AGREEMENT" if agrees else "DISAGREEMENT", row["file"], row["start_line"], row["target"], row["negative"]))
            print("ORACLE_RESOLUTION", name, results[-1][2], f"site={row['file']}:{row['start_line']} target={row['target']} negative={row['negative']} target_error={positive} negative_site_error={negative}", flush=True)
            # Swap the alleged definition with its actual same-name negative.
            # These are live compiler observations, not fabricated booleans.
            if agrees:
                assert not agreement(negative, positive), "mutated resolution claim escaped the oracle"
                caught_wrong_claim = True
                try:
                    agreement(positive, None)
                except AssertionError as error:
                    assert "negative control disabled" in str(error)
                    negative_disabled_caught = True
        assert caught_wrong_claim, "no passing pair available to challenge a wrong resolution claim"
        assert negative_disabled_caught, "negative-control deletion escaped the harness's own test"
        controls.extend([("resolution", "pass"), ("negative", "pass")])
        print("ORACLE_CONTROLS type=pass resolution=pass negative=pass wrong_claim_caught=true missing_negative_caught=true", flush=True)
    finally:
        scratch.restore()
        scratch.verify_originals()
        # Findings survive a later failed control; no disagreements disappear.
        with (directory / "results.tsv").open("w", newline="") as stream:
            writer = csv.writer(stream, delimiter="\t", lineterminator="\n")
            writer.writerow(("technique", "sample", "result", "file", "line", "claim", "control"))
            writer.writerows(results)
        (directory / "controls.tsv").write_text("".join(f"{k}\t{v}\n" for k, v in controls))
    disagreements = sum(r[2] == "DISAGREEMENT" for r in results)
    print(f"ORACLE_TOTAL agreements={len(results) - disagreements} disagreements={disagreements} techniques=3/3", flush=True)


if __name__ == "__main__":
    run(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
