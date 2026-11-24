//! Default P2379 ownership proof. One process, four capability modes, no admission.
//! The sealed guest table hashes `cases.rs`. A covered edit makes that load fail closed;
//! the same program bytes, pauses, and phase labels are then built for `guest::run`.
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use rum_runtime::cm740_probe::q25::{self, Half, Mode};
use syn::visit::Visit;

use super::oracle::fixtures::obligations::guest::{self, Program, RunFailure, RunObservation};
use super::oracle::obligations::bindings::TupleKey;
use super::oracle::obligations::content_digest;
use crate::{HostEvalOutcome, HostInputKind, HostInputRequest, HostTyped};

const PROGRAM_ID: &str =
    "C15.stringresultappend.P2379.AlreadyResumed.consumer_routes.route-original";
const SOURCE_SHA: &str = "e5e3a1ffb2d76ccefaef22f7c1a32ba8246e4ffd3933a1db1b21994a39dfd59b";
const SETUP_SHA: &str = "ae59ceca1fc118beea303570e170412c9457ea1148d6020f93568c6c4bc98933";
const RECORDS_SHA: &str = "7364f150437a0d395caa6843dabbb99a0c2500443faaa6ad8951f6af2eb1aa28";
const CLAUSE_SHA: &str = "3685fd041b6d77525b4da638f39b9228c01d7439bb1dfff302428842b0ed5495";
const IMPORTER_SHA: &str = "f2a919db543aa8976a12223d34c97724966ad8ad5409360604f01f4ff316c2e7";
const ANSWERS: &[(&str, &str)] = &[("prior", "prior"), ("finish", "\"X\"")];

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn observation_path() -> PathBuf {
    workspace()
        .parent()
        .unwrap()
        .join("logs/q25-fixture-observations.log")
}

fn emit(line: &str) {
    println!("{line}");
    let _ = std::io::stdout().flush();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(observation_path())
    {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("Q25_READ {path:?} {error}"))
}

fn keep_lines(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

fn call_ends_with(expr: &syn::Expr, name: &str) -> bool {
    let syn::Expr::Call(call) = expr else {
        return false;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

// syn 2.0.117 stores `self: Arc<Self>` as FnArg::Receiver (reference none,
// colon present), never as FnArg::Typed. Accept only that shape when the
// type tokens contain both Arc and Self. Every other receiver shape has its
// own rejection message.
fn note_ident(ident: &proc_macro2::Ident, arc: &mut bool, self_token: &mut bool) {
    let text = ident.to_string();
    if text == "Arc" {
        *arc = true;
    } else if text == "Self" {
        *self_token = true;
    }
}

fn walk_tokens(stream: proc_macro2::TokenStream, arc: &mut bool, self_token: &mut bool) {
    for tree in stream {
        match tree {
            proc_macro2::TokenTree::Ident(ident) => note_ident(&ident, arc, self_token),
            proc_macro2::TokenTree::Group(group) => walk_tokens(group.stream(), arc, self_token),
            _ => {}
        }
    }
}

fn tokens_carry_arc_and_self(ty: &syn::Type) -> bool {
    let mut arc = false;
    let mut self_token = false;
    walk_tokens(ty.to_token_stream(), &mut arc, &mut self_token);
    arc && self_token
}

fn classify_receiver(arg: &syn::FnArg) -> Result<(), String> {
    let syn::FnArg::Receiver(receiver) = arg else {
        return Err("not a receiver".to_string());
    };
    if receiver.reference.is_some() {
        let lifetime = receiver
            .reference
            .as_ref()
            .and_then(|pair| pair.1.as_ref())
            .map(|life| life.to_token_stream().to_string())
            .unwrap_or_default();
        let mutable = if receiver.mutability.is_some() {
            "mut "
        } else {
            ""
        };
        let gap = if lifetime.is_empty() { "" } else { " " };
        return Err(format!("reference receiver &{lifetime}{gap}{mutable}self"));
    }
    if receiver.colon_token.is_none() {
        return Err(if receiver.mutability.is_some() {
            "bare mut self".to_string()
        } else {
            "bare self".to_string()
        });
    }
    if receiver.mutability.is_some() {
        return Err("explicit mut receiver".to_string());
    }
    if tokens_carry_arc_and_self(&receiver.ty) {
        Ok(())
    } else {
        Err(format!(
            "wrong receiver type {}",
            receiver.ty.to_token_stream()
        ))
    }
}

fn snippet_resume_verdict(snippet: &str) -> String {
    let file =
        syn::parse_file(snippet).unwrap_or_else(|error| panic!("Q25_AST_FALSIFIER parse: {error}"));
    for item in &file.items {
        let syn::Item::Impl(block) = item else {
            continue;
        };
        for member in &block.items {
            let syn::ImplItem::Fn(function) = member else {
                continue;
            };
            if function.sig.ident != "resume" {
                continue;
            }
            let Some(argument) = function.sig.inputs.first() else {
                panic!("Q25_AST_FALSIFIER resume has no argument");
            };
            return match classify_receiver(argument) {
                Ok(()) => "accepted".to_string(),
                Err(message) => format!("rejected:{message}"),
            };
        }
    }
    panic!("Q25_AST_FALSIFIER snippet has no impl resume");
}

fn falsify_receiver_classifier() {
    let real = snippet_resume_verdict(
        "trait SuspensionFrame { fn resume(self: std::sync::Arc<Self>); }\nimpl SuspensionFrame for Frame { fn resume(self: std::sync::Arc<Self>) {} }\n",
    );
    let reference = snippet_resume_verdict(
        "trait SuspensionFrame { fn resume(&self); }\nimpl SuspensionFrame for Frame { fn resume(&self) {} }\n",
    );
    let wrong = snippet_resume_verdict(
        "trait SuspensionFrame { fn resume(self: std::sync::Arc<Other>); }\nimpl SuspensionFrame for Frame { fn resume(self: std::sync::Arc<Other>) {} }\n",
    );
    emit(&format!(
        "Q25_AST_FALSIFIER real={real} ampersand_self={reference} wrong_type={wrong}"
    ));
    if real != "accepted"
        || reference != "rejected:reference receiver &self"
        || !wrong.starts_with("rejected:wrong receiver type ")
        || !wrong.contains("Arc")
        || !wrong.contains("Other")
        || wrong.split_whitespace().any(|token| token == "Self")
    {
        panic!(
            "Q25_AST_FALSIFIER expected accepted / rejected &self / rejected wrong type, \
             got real={real} ampersand_self={reference} wrong_type={wrong}"
        );
    }
}

fn require_resume_shape() {
    let path =
        workspace().join("crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs");
    let text = read_text(&path);
    let file = syn::parse_file(&text)
        .unwrap_or_else(|error| panic!("Q25_AST parse resume_frames: {error}"));
    let mut found = false;
    for item in &file.items {
        let syn::Item::Impl(item) = item else {
            continue;
        };
        let self_ty = item.self_ty.to_token_stream().to_string();
        if !self_ty.contains("DriveStringCasesOutputResultSuspensionFrame") {
            continue;
        }
        let Some((_, trait_path, _)) = &item.trait_ else {
            continue;
        };
        if !trait_path
            .to_token_stream()
            .to_string()
            .contains("SuspensionFrame")
        {
            continue;
        }
        for member in &item.items {
            let syn::ImplItem::Fn(function) = member else {
                continue;
            };
            if function.sig.ident != "resume" {
                continue;
            }
            let Some(argument) = function.sig.inputs.first() else {
                panic!("Q25_AST resume receiver is missing");
            };
            if let Err(message) = classify_receiver(argument) {
                panic!("Q25_AST resume receiver rejected: {message}");
            }
            let Some(syn::Stmt::Local(local)) = function.block.stmts.first() else {
                panic!("Q25_AST resume first statement is not a local");
            };
            let Some(init) = &local.init else {
                panic!("Q25_AST resume first local has no initializer");
            };
            if !call_ends_with(&init.expr, "begin_concrete") {
                panic!(
                    "Q25_AST resume first call is {}",
                    init.expr.to_token_stream()
                );
            }
            found = true;
        }
    }
    if !found {
        panic!("Q25_AST missing DriveStringCasesOutputResultSuspensionFrame::resume");
    }
    emit("Q25_AST resume_frames.rs receiver=Arc<Self> first=begin_concrete");
}

fn require_clause() {
    let path = workspace().join("crates/rum-stdlib/tests/suspension_oracle/mutations/records.rs");
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("Q25_READ {path:?} {error}"));
    let text = String::from_utf8(bytes.clone())
        .unwrap_or_else(|error| panic!("Q25_UTF8 records.rs {error}"));
    let file_sha = content_digest(&bytes);
    if file_sha != RECORDS_SHA {
        panic!("Q25_CLAUSE records.rs sha={file_sha} expected={RECORDS_SHA}");
    }
    let lines = keep_lines(&text);
    if lines.len() < 366 {
        panic!("Q25_CLAUSE records.rs lines={}", lines.len());
    }
    let clause = lines[340..351].concat();
    let clause_sha = content_digest(clause.as_bytes());
    if clause_sha != CLAUSE_SHA {
        panic!("Q25_CLAUSE records.rs:341-351 sha={clause_sha} expected={CLAUSE_SHA}");
    }
    let importer = lines[282..305].concat();
    let importer_sha = content_digest(importer.as_bytes());
    if importer_sha != IMPORTER_SHA {
        panic!("Q25_CLAUSE records.rs:283-305 sha={importer_sha} expected={IMPORTER_SHA}");
    }
    let parsed =
        syn::parse_file(&text).unwrap_or_else(|error| panic!("Q25_AST parse records.rs: {error}"));
    let mut shape = NoHitShape::default();
    shape.visit_file(&parsed);
    if !shape.hit_before_arm || !shape.hits_empty || !shape.no_hit || !shape.identity_survivor {
        panic!(
            "Q25_AST records.rs shape hit_before_arm={} hits_empty={} no_hit={} identity_survivor={}",
            shape.hit_before_arm, shape.hits_empty, shape.no_hit, shape.identity_survivor
        );
    }
    if !clause.contains("hit_before_arm")
        || !clause.contains("hits.is_empty()")
        || !clause.contains("Rejection::NoHit")
    {
        panic!("Q25_CLAUSE records.rs:341-351 text missing the NoHit decision");
    }
    if !lines[351..366]
        .concat()
        .contains("Rejection::IdentitySurvivor")
    {
        panic!("Q25_CLAUSE records.rs:352-366 missing IdentitySurvivor");
    }
    emit(&format!(
        "Q25_CLAUSE records.rs:341-351 sha={clause_sha} shape=NoHit before=352-366"
    ));
    emit("Q25_IMPORTER records.rs:283-305 DEFERRED not-claimed");
}

#[derive(Default)]
struct NoHitShape {
    hit_before_arm: bool,
    hits_empty: bool,
    no_hit: bool,
    identity_survivor: bool,
}

impl<'ast> Visit<'ast> for NoHitShape {
    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if let syn::Member::Named(ident) = &node.member {
            if ident.to_string() == "hit_before_arm" {
                self.hit_before_arm = true;
            }
        }
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method.to_string() == "is_empty" {
            self.hits_empty = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        let text = node.to_token_stream().to_string();
        if text.contains("hit_before_arm") {
            self.hit_before_arm = true;
        }
        if text.contains("NoHit") {
            self.no_hit = true;
        }
        if text.contains("IdentitySurvivor") {
            self.identity_survivor = true;
        }
        syn::visit::visit_expr_path(self, node);
    }
}

fn static_rows(executed_p2379: bool) {
    let rows = [
        (
            "P0214",
            "crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs",
            false,
        ),
        (
            "P0216",
            "crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs",
            false,
        ),
        (
            "P1070",
            "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs",
            false,
        ),
        (
            "P2348",
            "crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs",
            false,
        ),
        (
            "P2354",
            "crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs",
            false,
        ),
        (
            "P2379",
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            true,
        ),
        (
            "P2381",
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            false,
        ),
        (
            "P2385",
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            false,
        ),
        (
            "P2387",
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            false,
        ),
        (
            "P2409",
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            false,
        ),
        (
            "P2411",
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            false,
        ),
        (
            "P2416",
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            false,
        ),
        (
            "P2418",
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            false,
        ),
    ];
    for (row, relative, selected) in rows {
        let path = workspace().join(relative);
        let text = read_text(&path);
        if !text.contains(&format!("\"{row}\"")) {
            panic!("Q25_STATIC missing {row} in {relative}");
        }
        let executed = selected && executed_p2379;
        emit(&format!(
            "Q25_STATIC row={row} executed={executed} path={relative} anchor=present"
        ));
    }
}

fn manual_program() -> Program {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/suspension_oracle/fixtures/obligations/programs");
    let source_bytes = std::fs::read(dir.join(format!("{SOURCE_SHA}.rum"))).expect("program bytes");
    let setup_bytes = std::fs::read(dir.join(format!("{SETUP_SHA}.rum"))).expect("setup bytes");
    let source_sha = content_digest(&source_bytes);
    let setup_sha = content_digest(&setup_bytes);
    if source_sha != SOURCE_SHA || setup_sha != SETUP_SHA {
        panic!("Q25_PROGRAM bytes source={source_sha} setup={setup_sha}");
    }
    let source = String::from_utf8(source_bytes).expect("program utf8");
    let setup = String::from_utf8(setup_bytes).expect("setup utf8");
    Program {
        id: PROGRAM_ID.to_string(),
        family: "string-result-append".to_string(),
        key: TupleKey([
            "P2379".to_string(),
            "AlreadyResumed".to_string(),
            "consumer_routes.route-original".to_string(),
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:83:::resume_collect_string_cases_from_index@138".to_string(),
            "StringCases.delayedRhs".to_string(),
        ]),
        source,
        setup,
        source_hash: SOURCE_SHA.to_string(),
        setup_hash: SETUP_SHA.to_string(),
        answers_hash: content_digest(b"InputString:7072696f72:7072696f72;Input:66696e697368:225822"),
        plan_hash: content_digest(PROGRAM_ID.as_bytes()),
        pauses: vec![
            (
                HostInputRequest {
                    kind: HostInputKind::InputString,
                    prompt: "prior".to_string(),
                    password: false,
                },
                "prior".to_string(),
            ),
            (
                HostInputRequest {
                    kind: HostInputKind::Input,
                    prompt: "finish".to_string(),
                    password: false,
                },
                "\"X\"".to_string(),
            ),
        ],
        trace: true,
        producer_route: "StringCases>resume_collect_string_cases_from_index>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation".to_string(),
        phases: vec![
            "prior-uncredited".to_string(),
            "P2379:string-result-append:operand-1".to_string(),
        ],
        canonical: None,
        paired: None,
    }
}

fn load_program() -> Program {
    match guest::programs() {
        Ok(programs) => match programs.get(PROGRAM_ID) {
            Some(program) => {
                emit(&format!("Q25_PROGRAMS loader=ok id={PROGRAM_ID}"));
                program.clone()
            }
            None => panic!("Q25_PROGRAMS loader=ok missing {PROGRAM_ID}"),
        },
        Err(error) => {
            emit(&format!(
                "Q25_PROGRAMS loader={error:?} fallback=manual reason=sealed_source_digest"
            ));
            manual_program()
        }
    }
}

fn strings_of(outcome: &HostEvalOutcome) -> Result<Vec<String>, String> {
    let HostEvalOutcome::Success(success) = outcome else {
        return Err(format!("not-success {outcome:?}"));
    };
    let HostTyped::List(items) = &success.value.typed else {
        return Err(format!(
            "text={} typed={:?}",
            success.value.text, success.value.typed
        ));
    };
    let mut values = Vec::new();
    for item in items {
        match item {
            HostTyped::Str(value) => values.push(value.clone()),
            other => return Err(format!("element {other:?} text={}", success.value.text)),
        }
    }
    Ok(values)
}

fn count_having(lines: &[&str], event: &str, extra: &str) -> usize {
    lines
        .iter()
        .filter(|line| line.contains(event) && (extra.is_empty() || line.contains(extra)))
        .count()
}

fn tagged<'a>(lines: &'a [String], mode: &str, half: &str, nonce: u64) -> Vec<&'a str> {
    let tag = format!("mode={mode} half={half} nonce={nonce} ");
    lines
        .iter()
        .filter(|line| line.contains(&tag))
        .map(String::as_str)
        .collect()
}

fn field_usize(line: &str, key: &str) -> Option<usize> {
    let needle = format!("{key}=");
    let rest = line.split(&needle).nth(1)?;
    let token = rest
        .split(|ch: char| ch.is_whitespace())
        .next()
        .unwrap_or("");
    let token = token.trim_end_matches(',');
    token.parse().ok()
}

struct HalfRun {
    observation: Result<RunObservation, RunFailure>,
    lines: Vec<String>,
}

fn run_half(program: &Program, mode: Mode, half: Half, nonce: u64) -> HalfRun {
    let answers: Vec<(&str, &str)> = program
        .pauses
        .iter()
        .map(|(request, answer)| (request.prompt.as_str(), answer.as_str()))
        .collect();
    let capability = q25::issue(
        mode,
        half,
        &program.source,
        &program.setup,
        &answers,
        0,
        nonce,
        &program.source_hash,
        &program.setup_hash,
    )
    .unwrap_or_else(|| panic!("Q25_ISSUE rejected mode={mode:?} half={half:?} nonce={nonce}"));
    let start = q25::snapshot_lines().len();
    q25::install(capability);
    let queued = matches!(half, Half::Queued);
    emit(&format!(
        "Q25_MARK mode={mode:?} half={half:?} nonce={nonce} begin"
    ));
    let observation = guest::run(program, queued);
    emit(&format!(
        "Q25_MARK mode={mode:?} half={half:?} nonce={nonce} end verdict={:?}",
        observation.as_ref().err()
    ));
    let lines = q25::snapshot_lines()[start..].to_vec();
    for line in &lines {
        emit(line);
    }
    if let Ok(seen) = &observation {
        emit(&format!(
            "Q25_OUTCOME mode={mode:?} half={half:?} text={} pauses={} native_entries={} native_resumes={} counters={}",
            match &seen.output {
                HostEvalOutcome::Success(success) => success.value.text.clone(),
                other => format!("{other:?}"),
            },
            seen.pauses.len(),
            seen.native.entries,
            seen.native.resumes,
            seen.counters.len()
        ));
    }
    HalfRun { observation, lines }
}

fn expect(failures: &mut Vec<String>, cond: bool, message: String) {
    if !cond {
        failures.push(message.clone());
        emit(&format!("Q25_EXPECT_FAIL {message}"));
    }
}

fn list_is(failures: &mut Vec<String>, label: &str, run: &HalfRun, want: &[&str]) {
    match &run.observation {
        Ok(seen) => match strings_of(&seen.output) {
            Ok(got) => expect(
                failures,
                got == want,
                format!("{label} list got={got:?} want={want:?} text_checked=typed"),
            ),
            Err(text) => expect(failures, false, format!("{label} list {text}")),
        },
        Err(error) => expect(failures, false, format!("{label} run={error:?}")),
    }
}

fn validate_is(
    failures: &mut Vec<String>,
    label: &str,
    program: &Program,
    unqueued: &HalfRun,
    queued: &HalfRun,
    want_ok: bool,
) {
    let (Ok(left), Ok(right)) = (&unqueued.observation, &queued.observation) else {
        expect(failures, false, format!("{label} validate skipped"));
        return;
    };
    let verdict = guest::validate(program, left, right);
    let public_same = left.output == right.output;
    let native_diff = left.counters != right.counters || left.native != right.native;
    emit(&format!(
        "Q25_VALIDATE mode={label} verdict={verdict:?} public_output_exact={public_same} native_difference={native_diff} observer=Strict"
    ));
    expect(
        failures,
        want_ok == verdict.is_ok(),
        format!("{label} validate got={verdict:?} want_ok={want_ok}"),
    );
    if want_ok {
        expect(
            failures,
            public_same,
            format!("{label} public_output_exact got={public_same}"),
        );
    } else {
        expect(
            failures,
            matches!(verdict, Err(RunFailure::Output)) && !public_same,
            format!(
                "{label} want Err(Output) and public_output_exact=false got={verdict:?} same={public_same}"
            ),
        );
    }
}

fn assert_counts(
    failures: &mut Vec<String>,
    label: &str,
    lines: &[&str],
    event: &str,
    extra: &str,
    want: usize,
) {
    let got = count_having(lines, event, extra);
    expect(
        failures,
        got == want,
        format!("{label} {event} {extra} got={got} want={want}"),
    );
}

fn assert_at_least(
    failures: &mut Vec<String>,
    label: &str,
    lines: &[&str],
    event: &str,
    extra: &str,
    want: usize,
) {
    let got = count_having(lines, event, extra);
    expect(
        failures,
        got >= want,
        format!("{label} {event} {extra} got={got} want_at_least={want}"),
    );
}

#[test]
fn q25_p2379_ownership_proof() {
    match std::env::var("RUM_CM740_OMIT") {
        Ok(value) if value == "P2379" => {}
        other => panic!("Q25_SELECTOR fail-closed got={other:?} want=Ok(\"P2379\")"),
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        emit(&format!("Q25_PANIC {info}"));
        for line in q25::snapshot_lines() {
            emit(&line);
        }
        previous(info);
    }));

    falsify_receiver_classifier();
    require_resume_shape();
    require_clause();
    emit(&format!(
        "Q25_ENV_MINT present={} guest_grant={}",
        q25::try_env_mint().is_some(),
        q25::guest_grant()
    ));
    if q25::guest_grant() || q25::try_env_mint().is_some() {
        panic!("Q25_ENV_MINT positive capability was minted from the environment or guest grant");
    }
    emit("Q25_OBSERVER label=Strict constructed=false");
    emit("Q25_PROOF non_admission=true importable_receipts=0");

    let program = load_program();
    if !program.trace || program.key.row() != "P2379" {
        panic!(
            "Q25_PROGRAM trace={} row={}",
            program.trace,
            program.key.row()
        );
    }
    let answers: Vec<(&str, &str)> = program
        .pauses
        .iter()
        .map(|(request, answer)| (request.prompt.as_str(), answer.as_str()))
        .collect();
    if answers != ANSWERS {
        panic!("Q25_PROGRAM answers={answers:?}");
    }
    emit(&format!(
        "Q25_PROGRAM id={} trace=true source_sha={} setup_sha={} pauses=2",
        program.id, program.source_hash, program.setup_hash
    ));

    let mut failures = Vec::new();
    let ordinary_u = run_half(&program, Mode::Ordinary, Half::Unqueued, 1);
    let ordinary_q = run_half(&program, Mode::Ordinary, Half::Queued, 2);
    let positive_u = run_half(&program, Mode::Positive, Half::Unqueued, 3);
    let positive_q = run_half(&program, Mode::Positive, Half::Queued, 4);
    let withheld_u = run_half(&program, Mode::Withheld, Half::Unqueued, 5);
    let withheld_q = run_half(&program, Mode::Withheld, Half::Queued, 6);
    let siblings_u = run_half(&program, Mode::Siblings, Half::Unqueued, 7);
    let siblings_q = run_half(&program, Mode::Siblings, Half::Queued, 8);

    let ran = [
        &ordinary_u,
        &ordinary_q,
        &positive_u,
        &positive_q,
        &withheld_u,
        &withheld_q,
        &siblings_u,
        &siblings_q,
    ]
    .iter()
    .all(|half| half.observation.is_ok());
    static_rows(ran);

    let ou = tagged(&ordinary_u.lines, "Ordinary", "Unqueued", 1);
    let oq = tagged(&ordinary_q.lines, "Ordinary", "Queued", 2);
    let pu = tagged(&positive_u.lines, "Positive", "Unqueued", 3);
    let pq = tagged(&positive_q.lines, "Positive", "Queued", 4);
    let wu = tagged(&withheld_u.lines, "Withheld", "Unqueued", 5);
    let wq = tagged(&withheld_q.lines, "Withheld", "Queued", 6);
    let su = tagged(&siblings_u.lines, "Siblings", "Unqueued", 7);
    let sq = tagged(&siblings_q.lines, "Siblings", "Queued", 8);

    list_is(&mut failures, "ordinary-unqueued", &ordinary_u, &["X", "X"]);
    list_is(&mut failures, "ordinary-queued", &ordinary_q, &["X", "X"]);
    list_is(&mut failures, "positive-unqueued", &positive_u, &["X"]);
    list_is(&mut failures, "positive-queued", &positive_q, &["X", "X"]);
    list_is(&mut failures, "withheld-unqueued", &withheld_u, &["X", "X"]);
    list_is(&mut failures, "withheld-queued", &withheld_q, &["X", "X"]);
    list_is(&mut failures, "siblings-unqueued", &siblings_u, &["X"]);
    list_is(&mut failures, "siblings-queued", &siblings_q, &["X", "X"]);
    validate_is(
        &mut failures,
        "ordinary",
        &program,
        &ordinary_u,
        &ordinary_q,
        true,
    );
    validate_is(
        &mut failures,
        "positive",
        &program,
        &positive_u,
        &positive_q,
        false,
    );
    validate_is(
        &mut failures,
        "withheld",
        &program,
        &withheld_u,
        &withheld_q,
        true,
    );
    validate_is(
        &mut failures,
        "siblings",
        &program,
        &siblings_u,
        &siblings_q,
        false,
    );

    assert_counts(&mut failures, "ordinary-unqueued", &ou, " PAUSE ", "", 2);
    assert_counts(&mut failures, "ordinary-queued", &oq, " PAUSE ", "", 0);
    assert_counts(&mut failures, "ordinary-unqueued", &ou, " ARM ", "", 0);
    assert_counts(&mut failures, "ordinary-queued", &oq, " ARM ", "", 0);
    assert_counts(
        &mut failures,
        "ordinary-unqueued",
        &ou,
        " APPEND ",
        "grant=true",
        0,
    );
    assert_counts(
        &mut failures,
        "ordinary-queued",
        &oq,
        " APPEND ",
        "grant=true",
        0,
    );
    assert_at_least(
        &mut failures,
        "ordinary-unqueued",
        &ou,
        " APPEND ",
        "grant=false",
        2,
    );

    assert_counts(
        &mut failures,
        "positive-unqueued",
        &pu,
        " ARM ",
        "prompt=\"finish\"",
        1,
    );
    assert_counts(&mut failures, "positive-unqueued", &pu, " ARM ", "", 1);
    assert_counts(
        &mut failures,
        "positive-unqueued",
        &pu,
        " APPEND ",
        "grant=true",
        1,
    );
    assert_at_least(
        &mut failures,
        "positive-unqueued",
        &pu,
        " APPEND ",
        "grant=false",
        1,
    );
    assert_counts(
        &mut failures,
        "positive-unqueued",
        &pu,
        " DISPATCH ",
        "grant=true",
        1,
    );
    assert_counts(&mut failures, "positive-queued", &pq, " ARM ", "", 0);
    assert_counts(
        &mut failures,
        "positive-queued",
        &pq,
        " APPEND ",
        "grant=true",
        0,
    );
    assert_counts(
        &mut failures,
        "positive-queued",
        &pq,
        " DISPATCH ",
        "grant=true",
        0,
    );
    assert_counts(
        &mut failures,
        "positive-unqueued",
        &pu,
        " HANDOFF ",
        "present",
        1,
    );
    assert_counts(
        &mut failures,
        "positive-unqueued",
        &pu,
        " REGISTER ",
        "commit=true",
        1,
    );
    assert_counts(
        &mut failures,
        "positive-unqueued",
        &pu,
        " REGISTER ",
        "strong=1",
        1,
    );

    let dispatch_strong = pu
        .iter()
        .find(|line| line.contains(" DISPATCH ") && line.contains("grant=true"))
        .and_then(|line| field_usize(line, "strong"));
    expect(
        &mut failures,
        dispatch_strong.is_some_and(|strong| strong >= 2),
        format!("positive-unqueued dispatch strong={dispatch_strong:?}"),
    );
    let registered = pu
        .iter()
        .find(|line| line.contains(" REGISTER ") && line.contains("commit=true"));
    let registered_addr = registered.and_then(|line| field_usize(line, "addr"));
    let disposed = pu.iter().any(|line| {
        line.contains(" DISPOSE ")
            && line.contains("tracked=true")
            && line.contains("weak_upgrade=false")
            && registered_addr.is_some_and(|addr| line.contains(&format!("addr={addr}")))
    });
    expect(
        &mut failures,
        disposed,
        format!("positive-unqueued dispose weak_upgrade=false addr={registered_addr:?}"),
    );
    let append_grants: Vec<bool> = pu
        .iter()
        .filter(|line| line.contains(" APPEND "))
        .map(|line| line.contains("grant=true"))
        .collect();
    expect(
        &mut failures,
        append_grants.iter().filter(|grant| **grant).count() == 1
            && append_grants
                .iter()
                .position(|grant| *grant)
                .is_some_and(|index| append_grants.iter().skip(index + 1).any(|grant| !grant)),
        format!("positive-unqueued second append denied grants={append_grants:?}"),
    );

    assert_counts(&mut failures, "withheld-unqueued", &wu, " PAUSE ", "", 2);
    assert_counts(&mut failures, "withheld-queued", &wq, " PAUSE ", "", 0);
    assert_counts(
        &mut failures,
        "withheld-unqueued",
        &wu,
        " ARM ",
        "prompt=\"finish\"",
        1,
    );
    assert_counts(&mut failures, "withheld-unqueued", &wu, " ARM ", "", 1);
    assert_counts(&mut failures, "withheld-queued", &wq, " ARM ", "", 0);
    assert_counts(
        &mut failures,
        "withheld-unqueued",
        &wu,
        " APPEND ",
        "grant=true",
        0,
    );
    assert_counts(
        &mut failures,
        "withheld-queued",
        &wq,
        " APPEND ",
        "grant=true",
        0,
    );
    assert_at_least(
        &mut failures,
        "withheld-unqueued",
        &wu,
        " APPEND ",
        "grant=false",
        2,
    );
    assert_counts(
        &mut failures,
        "withheld-unqueued",
        &wu,
        " REGISTER ",
        "commit=false",
        1,
    );
    assert_counts(
        &mut failures,
        "withheld-unqueued",
        &wu,
        " REGISTER ",
        "commit=true",
        0,
    );
    assert_counts(
        &mut failures,
        "withheld-unqueued",
        &wu,
        " DISPATCH ",
        "grant=true",
        0,
    );
    assert_counts(
        &mut failures,
        "withheld-queued",
        &wq,
        " DISPATCH ",
        "grant=true",
        0,
    );
    assert_counts(
        &mut failures,
        "withheld-unqueued",
        &wu,
        " HANDOFF ",
        "present",
        1,
    );
    if let (Ok(reference_u), Ok(reference_q), Ok(held_u), Ok(held_q)) = (
        &ordinary_u.observation,
        &ordinary_q.observation,
        &withheld_u.observation,
        &withheld_q.observation,
    ) {
        expect(
            &mut failures,
            reference_u.output == held_u.output && reference_q.output == held_q.output,
            "withheld outcomes differ from ordinary".to_string(),
        );
        expect(
            &mut failures,
            reference_u.native == held_u.native
                && reference_q.native == held_q.native
                && reference_u.counters == held_u.counters
                && reference_q.counters == held_q.counters,
            format!(
                "withheld native differs ordinary_u={:?} withheld_u={:?} ordinary_q={:?} withheld_q={:?}",
                reference_u.native, held_u.native, reference_q.native, held_q.native
            ),
        );
    }
    let withheld_hits =
        count_having(&wu, " APPEND ", "grant=true") + count_having(&wq, " APPEND ", "grant=true");
    let withheld_armed = count_having(&wu, " ARM ", "prompt=\"finish\"") == 1;
    let rejection = if withheld_armed && withheld_hits == 0 {
        "NoHit"
    } else if !withheld_armed {
        "Unarmed"
    } else {
        "other"
    };
    emit(&format!(
        "Q25_REJECTION withheld={rejection} clause=records.rs:344-350 before=352-366 armed={withheld_armed} hits={withheld_hits}"
    ));
    expect(
        &mut failures,
        rejection == "NoHit",
        format!("withheld rejection={rejection}"),
    );

    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " SIBLINGS ",
        "distinct=true",
        1,
    );
    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " SIBLINGS ",
        "fields_equal=true",
        1,
    );
    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " SIBLING_FORWARD ",
        "label=B",
        1,
    );
    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " SIBLING_FORWARD ",
        "label=C",
        1,
    );
    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " SIBLING_FORWARD ",
        "reject=true",
        2,
    );
    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " SIBLING_REVERSE ",
        "reject=true",
        2,
    );
    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " SIBLING_SLOT ",
        "survived=true",
        1,
    );
    assert_counts(
        &mut failures,
        "siblings-unqueued",
        &su,
        " APPEND ",
        "grant=true",
        1,
    );
    assert_counts(
        &mut failures,
        "siblings-queued",
        &sq,
        " APPEND ",
        "grant=true",
        0,
    );
    assert_counts(&mut failures, "siblings-queued", &sq, " ARM ", "", 0);

    let replay_start = q25::snapshot_lines().len();
    q25::watch_pairs(true);
    for (queued, label) in [
        (false, "open"),
        (true, "consume"),
        (true, "replay"),
        (false, "replace"),
        (false, "replace-again"),
    ] {
        rum_runtime::cm740_probe::begin(queued, "Q25-REPLAY");
        rum_runtime::cm740_probe::end();
        emit(&format!("Q25_REPLAY_STEP {label} queued={queued}"));
    }
    let replay: Vec<String> = q25::snapshot_lines()[replay_start..]
        .iter()
        .filter(|line| line.contains("program=\"Q25-REPLAY\"") && line.contains("BEGIN queued="))
        .cloned()
        .collect();
    for line in &replay {
        emit(line);
    }
    let replay_flags: Vec<(bool, bool)> = replay
        .iter()
        .map(|line| {
            (
                line.contains("queued=true"),
                line.contains("episode_present=true"),
            )
        })
        .collect();
    expect(
        &mut failures,
        replay_flags.len() == 5
            && replay_flags.first() == Some(&(false, true))
            && replay_flags.get(1) == Some(&(true, true))
            && replay_flags.get(2) == Some(&(true, false))
            && replay_flags.get(3) == Some(&(false, true)),
        format!("replay-pair flags={replay_flags:?}"),
    );

    let negative_start = q25::snapshot_lines().len();
    q25::exercise_negatives();
    q25::clause_note();
    let negatives = q25::snapshot_lines()[negative_start..].to_vec();
    for line in &negatives {
        emit(line);
    }
    for needle in [
        "NEGATIVE case=wrong-kind reject=true",
        "NEGATIVE case=wrong-front reject=true",
        "NEGATIVE case=invalid-generation reject=true",
        "NEGATIVE case=stale-address dead_weak=true reject=true",
        "NEGATIVE case=stale-address new_issue reject=true",
        "NEGATIVE case=second-take reject=true",
        "NEGATIVE case=wrong-source reject=true",
        "NEGATIVE case=cross-run reject=true",
        "NEGATIVE case=cross-domain guest_grant=false env_mint=false",
        "NEGATIVE case=nested reject=true",
        "NEGATIVE case=return-substitution reject=true",
        "NEGATIVE case=early-exit reject=true",
        "NEGATIVE case=wrong-phase reject=true",
        "NEGATIVE case=wrong-queued-operation queued_hit=0",
        "NEGATIVE case=abandon-or-cancel",
        "weak_owner_dropped=true",
        "IMPORTER records.rs:283-305 DEFERRED not-claimed",
    ] {
        expect(
            &mut failures,
            negatives.iter().any(|line| line.contains(needle)),
            format!("missing negative {needle}"),
        );
    }

    emit(&format!(
        "Q25_PROOF_RESULT failures={} non_admission=true importable_receipts=0 rows_closed=0",
        failures.len()
    ));
    if !failures.is_empty() {
        panic!("Q25_PROOF_FAIL {}", failures.join(" | "));
    }
    emit("Q25_PROOF_OK non_admission=true importable_receipts=0");
}
