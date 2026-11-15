//! Only the executed-run adapter constructs receipts. Declarations and proof
//! certificates have no conversion into this type.
use super::bindings::{BoundBag, BoundPlan, RequiredBag, TupleKey};
use crate::oracle::fixtures::obligations::{
    boundaries::{self, LifecycleObservation},
    guest::{self, RunFailure, RunObservation},
};
use std::{collections::BTreeSet, sync::OnceLock};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunIdentity {
    pub feature: &'static str,
    pub build: String,
    pub switches: Vec<(String, String)>,
}

impl RunIdentity {
    pub fn planned(plan: &BoundPlan) -> Self {
        let mut identity = if feature_lane(plan) {
            Self::configured_lane().expect("P0543 requires its authenticated build lane")
        } else {
            Self::current()
        };
        identity.switches = super::schedule::switches(profile_off(plan));
        identity
    }
    pub fn current() -> Self {
        static BUILD: OnceLock<String> = OnceLock::new();
        let hash = BUILD
            .get_or_init(|| {
                super::content_digest(&std::fs::read(std::env::current_exe().unwrap()).unwrap())
            })
            .clone();
        let lane = std::env::var("RUM_CM739_FEATURE_LANE").as_deref() == Ok("p0543-unchecked");
        Self {
            feature: if lane {
                "p0543-unchecked"
            } else if cfg!(feature = "hash") {
                "default"
            } else {
                "no-default"
            },
            build: if lane {
                Self::lane_build(&hash).expect("authenticated P0543 build identity")
            } else {
                hash
            },
            switches: std::env::vars()
                .filter(|(name, _)| name.starts_with("RUM_PROFILE_"))
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        }
    }

    fn lane_build(binary_hash: &str) -> Result<String, RunFailure> {
        let tree = std::env::var("RUM_CM739_LANE_TREE").map_err(|_| RunFailure::Program)?;
        let manifests =
            std::env::var("RUM_CM739_LANE_MANIFEST_DIFF").map_err(|_| RunFailure::Program)?;
        let expected =
            std::env::var("RUM_CM739_LANE_BINARY_SHA256").map_err(|_| RunFailure::Program)?;
        if expected != binary_hash
            || [&tree, &manifests, &expected]
                .iter()
                .any(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(RunFailure::Program);
        }
        Ok(format!(
            "sha256={binary_hash};tree={tree};manifest-diff={manifests};profile=release;codegen-units=1"
        ))
    }

    fn configured_lane() -> Result<Self, RunFailure> {
        static LANE: OnceLock<Result<RunIdentity, RunFailure>> = OnceLock::new();
        LANE.get_or_init(|| {
            let path = std::env::var_os("RUM_CM739_LANE_BINARY").ok_or(RunFailure::Program)?;
            let bytes = std::fs::read(path).map_err(|_| RunFailure::Program)?;
            Ok(Self {
                feature: "p0543-unchecked",
                build: Self::lane_build(&super::content_digest(&bytes))?,
                switches: super::schedule::switches(false),
            })
        })
        .clone()
    }
}

pub fn feature_lane(plan: &BoundPlan) -> bool {
    plan.key.row() == "P0543"
}

pub fn profile_off(plan: &BoundPlan) -> bool {
    plan.key.0[3]
        .split(';')
        .any(|field| field == "PROFILED=false")
}

#[derive(Clone, Debug)]
enum Observations {
    Local {
        resumed: RunObservation,
        queued: RunObservation,
        lifecycle: Vec<LifecycleObservation>,
    },
    Child {
        raw: [String; 3],
        actual_requests: usize,
        lifecycle: usize,
    },
}

#[derive(Clone, Debug)]
pub struct ExecutionReceipt {
    key: TupleKey,
    program: String,
    program_hash: String,
    setup_hash: String,
    answers_hash: String,
    plan_hash: String,
    source_identity: String,
    identity: RunIdentity,
    planned_requests: usize,
    observed_channels: BTreeSet<&'static str>,
    observations: Observations,
    raw_references: Vec<String>,
    paired: Option<Box<ExecutionReceipt>>,
}

pub type ReceiptBag = Vec<ExecutionReceipt>;

#[test]
fn cm_paired_receipt_negative_controls() {
    if !super::schedule::profile_child(
        "oracle::obligations::receipts::cm_paired_receipt_negative_controls",
    ) {
        return;
    }
    let required = super::bindings::ActiveRequiredBag::committed()
        .unwrap()
        .cohort(&["P0256"]);
    let bound = super::bindings::bind(
        &required,
        &super::bindings::FrozenBindings::committed()
            .unwrap()
            .cohort(&["P0256"]),
    )
    .unwrap();
    let plan = bound
        .plans
        .iter()
        .find(|p| {
            p.key.variant() == "Middle"
                && p.key.context() == "same_type_error_finish.explicit-cycle-controls-required"
        })
        .unwrap();
    let identity = RunIdentity::planned(plan);
    let receipt = execute(plan, &identity).unwrap();
    receipt.assert_identity(plan, &identity);
    let mut missing = receipt.clone();
    missing.paired = None;
    let mut wrong = receipt.clone();
    wrong.paired.as_mut().unwrap().program.push_str(":wrong");
    for (name, changed, predicate) in [
        ("missing-paired-run", missing, "paired receipt multiplicity"),
        ("wrong-paired-run", wrong, "executed program identity"),
    ] {
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            changed.assert_identity(plan, &identity);
        }))
        .unwrap_err();
        let message = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied())
            .unwrap();
        assert!(message.contains(predicate), "wrong predicate: {message}");
        println!("CM PAIRED FALSIFIER PASS name={name} predicate={predicate}");
    }
    println!("CM PAIRED RECEIPT PASS keys=1 constituent_programs=2 sessions=fresh multiplicity=1");
}

#[test]
fn cm_three_constituent_receipt_negative_controls() {
    if !super::schedule::profile_child(
        "oracle::obligations::receipts::cm_three_constituent_receipt_negative_controls",
    ) {
        return;
    }
    let required = super::bindings::ActiveRequiredBag::committed()
        .unwrap()
        .cohort(&["P0455"]);
    let bound = super::bindings::bind(
        &required,
        &super::bindings::FrozenBindings::committed()
            .unwrap()
            .cohort(&["P0455"]),
    )
    .unwrap();
    let plan = bound
        .plans
        .iter()
        .find(|p| {
            p.key.variant() == "Middle" && p.key.context() == "consumer_routes.route-original"
        })
        .unwrap();
    let identity = RunIdentity::planned(plan);
    let receipt = execute(plan, &identity).unwrap();
    receipt.assert_identity(plan, &identity);
    assert!(receipt.paired.as_ref().unwrap().paired.is_some());
    let mut missing_second = receipt.clone();
    missing_second.paired = missing_second.paired.take().unwrap().paired;
    let mut missing_third = receipt.clone();
    missing_third.paired.as_mut().unwrap().paired = None;
    let mut duplicate_second = receipt.clone();
    let mut duplicate = duplicate_second.paired.as_ref().unwrap().clone();
    duplicate.paired = None;
    duplicate_second.paired.as_mut().unwrap().paired = Some(duplicate);
    let mut wrong_third = receipt.clone();
    wrong_third
        .paired
        .as_mut()
        .unwrap()
        .paired
        .as_mut()
        .unwrap()
        .plan_hash
        .push_str(":wrong");
    for (name, changed, predicate) in [
        (
            "missing-second-run",
            missing_second,
            "executed program identity",
        ),
        (
            "missing-third-run",
            missing_third,
            "paired receipt multiplicity",
        ),
        (
            "duplicate-second-as-third",
            duplicate_second,
            "executed program identity",
        ),
        (
            "wrong-third-fingerprint",
            wrong_third,
            "executed program fingerprints",
        ),
    ] {
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            changed.assert_identity(plan, &identity);
        }))
        .unwrap_err();
        let message = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied())
            .unwrap();
        assert!(message.contains(predicate), "wrong predicate: {message}");
        println!("CM CONSTITUENT FALSIFIER PASS name={name} predicate={predicate}");
    }
    println!(
        "CM CONSTITUENT RECEIPT PASS keys=1 constituent_programs=3 sessions=fresh multiplicity=1"
    );
}

#[test]
fn cm_declared_duration_negative_controls() {
    use crate::{HostEvalOutcome, HostStructuredOutputKind, HostTyped};
    if !super::schedule::profile_child(
        "oracle::obligations::receipts::cm_declared_duration_negative_controls",
    ) {
        return;
    }
    let programs = guest::programs().unwrap();
    let p = programs
        .values()
        .find(|p| {
            p.key.row() == "P0515"
                && p.canonical
                    .as_ref()
                    .is_some_and(|f| f.0.id == "E1.2079.phase_observation.trace")
        })
        .unwrap();
    let resumed = guest::run(p, false).unwrap();
    let queued = guest::run(p, true).unwrap();
    guest::validate(p, &resumed, &queued).unwrap();
    println!(
        "CM DURATION RAW key={} resumed={resumed:?} queued={queued:?}",
        p.key.line()
    );

    fn change_value(out: &mut HostEvalOutcome, index: usize, replacement: HostTyped) {
        let HostEvalOutcome::Success(out) = out else {
            panic!("timing success")
        };
        let HostTyped::List(items) = &mut out.value.typed else {
            panic!("timing list")
        };
        items[index] = replacement;
        out.value.text = out
            .value
            .typed
            .to_expr(&rum_runtime::Session::new())
            .unwrap()
            .to_string();
        for item in &mut out.outputs {
            if item.kind == HostStructuredOutputKind::Result {
                item.text.clone_from(&out.value.text);
            }
        }
    }
    let mut sibling = resumed.clone();
    change_value(&mut sibling.output, 1, HostTyped::Integer(74));
    let mut print = resumed.clone();
    let HostEvalOutcome::Success(out) = &mut print.output else {
        unreachable!()
    };
    out.prints.push("changed non-duration print".into());
    let mut trace = resumed.clone();
    let HostEvalOutcome::Success(out) = &mut trace.output else {
        unreachable!()
    };
    assert!(out.trace.pop().is_some());
    let index = out
        .outputs
        .iter()
        .rposition(|o| o.kind == HostStructuredOutputKind::Trace)
        .unwrap();
    out.outputs.remove(index);
    for (name, changed) in [
        ("sibling", sibling),
        ("print", print),
        ("trace-event", trace),
    ] {
        assert_eq!(
            guest::validate(p, &changed, &queued),
            Err(RunFailure::Output),
            "{name}"
        );
        println!("CM DURATION FALSIFIER PASS name={name} predicate=Output");
    }
    let mut negative = resumed.clone();
    change_value(&mut negative.output, 0, HostTyped::Integer(-1));
    let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        guest::validate(p, &negative, &queued)
    }))
    .unwrap_err();
    let message = error
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| error.downcast_ref::<&str>().copied())
        .unwrap();
    assert!(
        message.contains("negative elapsed duration"),
        "wrong predicate: {message}"
    );
    println!(
        "CM DURATION FALSIFIER PASS name=negative-duration predicate=negative elapsed duration"
    );
    println!(
        "CM DURATION COMPARISON PASS declared_paths=[[0]] raw_preserved=true other_channels=exact"
    );
}

impl ExecutionReceipt {
    pub fn key(&self) -> &TupleKey {
        &self.key
    }
    pub fn observations(&self) -> (&RunObservation, &RunObservation, &[LifecycleObservation]) {
        match &self.observations {
            Observations::Local {
                resumed,
                queued,
                lifecycle,
            } => (resumed, queued, lifecycle),
            Observations::Child { .. } => {
                panic!("validator controls require their local actual run")
            }
        }
    }
    pub fn actual_requests(&self) -> usize {
        let own = match &self.observations {
            Observations::Local { resumed, .. } => resumed.pauses.len(),
            Observations::Child {
                actual_requests, ..
            } => *actual_requests,
        };
        own + self.paired.as_ref().map_or(0, |p| p.actual_requests())
    }
    pub fn lifecycle_count(&self) -> usize {
        let own = match &self.observations {
            Observations::Local { lifecycle, .. } => lifecycle.len(),
            Observations::Child { lifecycle, .. } => *lifecycle,
        };
        own + self.paired.as_ref().map_or(0, |p| p.lifecycle_count())
    }
    pub fn assert_identity(&self, plan: &BoundPlan, identity: &RunIdentity) {
        let programs = guest::programs().unwrap();
        let p = &programs[&plan.program];
        self.assert_program(p, &plan.source_identity, identity);
        assert_eq!(&self.key, &plan.key);
    }

    fn assert_program(&self, p: &guest::Program, source_identity: &str, identity: &RunIdentity) {
        assert_eq!(
            (&self.key, &self.program, self.source_identity.as_str()),
            (&p.key, &p.id, source_identity),
            "executed program identity"
        );
        assert_eq!(
            (
                &self.program_hash,
                &self.setup_hash,
                &self.answers_hash,
                &self.plan_hash
            ),
            (&p.source_hash, &p.setup_hash, &p.answers_hash, &p.plan_hash),
            "executed program fingerprints"
        );
        assert_eq!(&self.identity, identity);
        assert_eq!(self.planned_requests, p.pauses.len());
        assert_eq!(self.observed_channels, channels());
        assert_eq!(self.raw_references.len(), 4);
        assert_eq!(
            self.paired.is_some(),
            p.paired.is_some(),
            "paired receipt multiplicity"
        );
        if let (Some(receipt), Some(program)) = (&self.paired, &p.paired) {
            receipt.assert_program(program, source_identity, identity);
        }
        match &self.observations {
            Observations::Local {
                resumed,
                queued,
                lifecycle,
            } => {
                guest::validate(p, resumed, queued).unwrap();
                boundaries::validate(p, lifecycle).unwrap();
            }
            Observations::Child { raw, .. } => {
                for (text, hash) in raw.iter().zip(&self.raw_references[1..]) {
                    assert_eq!(&super::content_digest(text.as_bytes()), hash);
                }
                assert_eq!(self.actual_requests(), p.pauses.len());
                assert_eq!(self.lifecycle_count(), 2 * p.subject_requests());
            }
        }
    }
}

fn channels() -> BTreeSet<&'static str> {
    [
        "value",
        "error",
        "termination",
        "prints",
        "messages",
        "trace",
        "outputs",
        "state",
        "requests",
        "native",
        "lifecycle",
        "queue",
        "fresh",
    ]
    .into_iter()
    .collect()
}

pub fn execute(plan: &BoundPlan, identity: &RunIdentity) -> Result<ExecutionReceipt, RunFailure> {
    if identity != &RunIdentity::current() || identity != &RunIdentity::planned(plan) {
        return Err(RunFailure::Program);
    }
    let programs = guest::programs().map_err(|_| RunFailure::Program)?;
    let p = programs.get(&plan.program).ok_or(RunFailure::Program)?;
    if p.key != plan.key
        || ![
            "empty-prefix-callback-value-charge",
            "retained-callback-prefix-restoration",
            "sow-match-recording",
            "trace-filter-emission",
            "exact-default-guard-body-evaluation",
            "direct-default-fallback",
            "direct-default-option",
            "string-result-append",
            "linear-literal-output",
            "delete-missing-target-tails",
            "traversal-accumulation",
            "association-store",
            "count-targets",
            "differences-row-order-tails",
            "lazy-callback-completion",
            "named-pattern-binding",
            "schema-validation-finish",
            "definition-inner-match-guard-tail",
            "exception-match-and-guard-dispatch",
            "held-store-and-destructuring-tails",
            "option-value-list-tail",
            "replace-repeated-matcher-pause",
            "replace-repeated-pass-finishes",
            "returned-request-accounting-and-maps",
            "exception-rhs-guard-dispatch",
            "matched-rhs-accounting",
            "returned-catch-and-default-owners",
            "status-pure-property-value-tail",
            "accepted-request-terminal-retirement",
            "request-checkpoint-current-recheck",
            "catch-body-result-dispatch",
            "host-call-structured-result-output",
            "jupyter-segment-cancelled-result-normalization",
            "fixedpoint-pass-publication",
            "cm3-carrier",
            "cm3-structured",
            "cm3-optional",
            "cm3-repaired",
            "cm3-set-rhs",
            "cm3-prepared",
            "resuming-driver-saved-state-restoration",
            "unchecked-builtin-feature-driver",
            "between-request-evaluation-retirement",
            "builtin-checked-pause-accounting",
            "consumer",
            "option-readers",
            "accepted-ownvalue-body-evaluation",
            "normalized-rule-guard-reconstruction",
            "compiled-matcher-adopted-accounting",
            "compiled-matcher-counter-revisit",
            "head",
            "readonly-second-target-suffix",
            "sequence-choice-tail",
            "sequence-choice-completion",
            "rewrite-value-adoption-and-binding-substitution",
            "upvalue-matcher-binding-adoption",
            "default-returned",
            "diagnostic-native",
            "c31-synchronous",
            "normalized-rule-list-emission",
            "traversal-settings-adoption",
            "catch-tag-match-carriers",
            "rule-lhs-child-parent-acceptance",
            "matcher-pattern-adoption",
            "guarded-replace-part-targets",
            "count-like-traversal-completion",
            "position-traversal-completion",
            "owned-guard-suffix-and-deferred-registration",
        ]
        .contains(&p.family.as_str())
    {
        return Err(RunFailure::Program);
    }
    let receipt = execute_program(p, plan, identity)?;
    println!("CM EXECUTION RAW {receipt:?}");
    Ok(receipt)
}

fn execute_program(
    p: &guest::Program,
    plan: &BoundPlan,
    identity: &RunIdentity,
) -> Result<ExecutionReceipt, RunFailure> {
    let resumed = guest::run(p, false)?;
    let queued = guest::run(p, true)?;
    if let Err(error) = guest::validate(p, &resumed, &queued) {
        eprintln!(
            "CM EXECUTION REJECT key={} predicate={error:?} resumed={resumed:?} queued={queued:?}",
            plan.key.line()
        );
        return Err(error);
    }
    let lifecycle = boundaries::run(p)?;
    let raw_references = vec![
        plan.evidence.clone(),
        super::content_digest(format!("{resumed:?}").as_bytes()),
        super::content_digest(format!("{queued:?}").as_bytes()),
        super::content_digest(format!("{lifecycle:?}").as_bytes()),
    ];
    let paired = p
        .paired
        .as_ref()
        .map(|program| execute_program(program, plan, identity).map(Box::new))
        .transpose()?;
    Ok(ExecutionReceipt {
        key: plan.key.clone(),
        program: p.id.clone(),
        program_hash: p.source_hash.clone(),
        setup_hash: p.setup_hash.clone(),
        answers_hash: p.answers_hash.clone(),
        plan_hash: p.plan_hash.clone(),
        source_identity: plan.source_identity.clone(),
        identity: identity.clone(),
        planned_requests: p.pauses.len(),
        observed_channels: channels(),
        observations: Observations::Local {
            resumed,
            queued,
            lifecycle,
        },
        raw_references,
        paired,
    })
}

fn hex(text: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(text.len() * 2);
    for byte in text.bytes() {
        write!(&mut encoded, "{byte:02x}").unwrap();
    }
    encoded
}

fn unhex(text: &str) -> Result<String, RunFailure> {
    if !text.is_ascii() || text.len() % 2 != 0 {
        return Err(RunFailure::Program);
    }
    let bytes = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(|_| RunFailure::Program))
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|_| RunFailure::Program)
}

/// Export only a receipt just produced by this live child adapter. No boolean
/// or historical file is accepted as execution evidence by the parent.
pub fn export_live(receipt: &ExecutionReceipt, nonce: &str) {
    let (resumed, queued, lifecycle) = receipt.observations();
    let cells = vec![
        nonce.to_string(),
        receipt.key.line(),
        receipt.program.clone(),
        receipt.program_hash.clone(),
        receipt.setup_hash.clone(),
        receipt.answers_hash.clone(),
        receipt.plan_hash.clone(),
        receipt.source_identity.clone(),
        receipt.identity.feature.into(),
        receipt.identity.build.clone(),
        format!("{:?}", receipt.identity.switches),
        receipt.planned_requests.to_string(),
        receipt.actual_requests().to_string(),
        receipt.lifecycle_count().to_string(),
        receipt
            .observed_channels
            .iter()
            .copied()
            .collect::<Vec<_>>()
            .join(";"),
        receipt.raw_references.join("\n"),
        format!("{resumed:?}"),
        format!("{queued:?}"),
        format!("{lifecycle:?}"),
    ];
    println!(
        "CM739_WIRE\t{}",
        cells.iter().map(|s| hex(s)).collect::<Vec<_>>().join("\t")
    );
}

/// This constructor's input is the stdout of a newly executed copy of this
/// exact test binary. The child validates complete typed observations locally;
/// the parent preserves those raw observations and checks every receipt field.
pub fn execute_off_group(plans: &[BoundPlan]) -> Result<ReceiptBag, RunFailure> {
    execute_child_group(plans, false)
}

pub fn execute_p0543_group(plans: &[BoundPlan]) -> Result<ReceiptBag, RunFailure> {
    execute_child_group(plans, true)
}

fn execute_child_group(plans: &[BoundPlan], lane: bool) -> Result<ReceiptBag, RunFailure> {
    if plans.is_empty() {
        return Ok(Vec::new());
    }
    let mut expected = std::collections::BTreeMap::new();
    for plan in plans {
        if (if lane {
            !feature_lane(plan) || profile_off(plan)
        } else {
            !profile_off(plan) || feature_lane(plan)
        }) || expected.insert(plan.key.line(), plan).is_some()
        {
            return Err(RunFailure::Program);
        }
    }
    let nonce = format!(
        "{}:{}",
        std::process::id(),
        super::content_digest(
            expected
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes()
        )
    );
    let output = if lane {
        super::schedule::p0543_worker(&nonce)
    } else {
        super::schedule::off_worker(&nonce)
    }
    .map_err(|_| RunFailure::Program)?;
    let stdout = String::from_utf8(output.stdout).map_err(|_| RunFailure::Program)?;
    print!("{stdout}");
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(RunFailure::Program);
    }
    let end = format!("CM739_WIRE_END\t{nonce}\t{}", expected.len());
    if stdout.lines().filter(|l| *l == end).count() != 1 {
        return Err(RunFailure::Program);
    }
    let mut seen = BTreeSet::new();
    let mut receipts = Vec::new();
    let programs = guest::programs().map_err(|_| RunFailure::Program)?;
    for line in stdout
        .lines()
        .filter_map(|l| l.strip_prefix("CM739_WIRE\t"))
    {
        let c = line.split('\t').map(unhex).collect::<Result<Vec<_>, _>>()?;
        if c.len() != 19 || c[0] != nonce || !seen.insert(c[1].clone()) {
            return Err(RunFailure::Program);
        }
        let plan = expected.get(&c[1]).ok_or(RunFailure::Program)?;
        let p = &programs[&plan.program];
        let identity = RunIdentity::planned(plan);
        let lifecycle = 2 * p.subject_requests();
        let refs: Vec<_> = c[15].split('\n').map(str::to_string).collect();
        let raw = [c[16].clone(), c[17].clone(), c[18].clone()];
        if c[2] != p.id
            || c[3] != p.source_hash
            || c[4] != p.setup_hash
            || c[5] != p.answers_hash
            || c[6] != p.plan_hash
            || c[7] != plan.source_identity
            || c[8] != identity.feature
            || c[9] != identity.build
            || c[10] != format!("{:?}", identity.switches)
            || c[11] != p.pauses.len().to_string()
            || c[12] != p.pauses.len().to_string()
            || c[13] != lifecycle.to_string()
            || c[14] != channels().into_iter().collect::<Vec<_>>().join(";")
            || refs.len() != 4
            || refs[0] != plan.evidence
            || raw
                .iter()
                .zip(refs.iter().skip(1))
                .any(|(s, h)| super::content_digest(s.as_bytes()) != *h)
        {
            return Err(RunFailure::Program);
        }
        receipts.push(ExecutionReceipt {
            key: plan.key.clone(),
            program: p.id.clone(),
            program_hash: p.source_hash.clone(),
            setup_hash: p.setup_hash.clone(),
            answers_hash: p.answers_hash.clone(),
            plan_hash: p.plan_hash.clone(),
            source_identity: plan.source_identity.clone(),
            identity,
            planned_requests: p.pauses.len(),
            observed_channels: channels(),
            observations: Observations::Child {
                raw,
                actual_requests: p.pauses.len(),
                lifecycle,
            },
            raw_references: refs,
            paired: None,
        });
    }
    if seen != expected.keys().cloned().collect() {
        return Err(RunFailure::Program);
    }
    Ok(receipts)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactError {
    DuplicateRequired,
    DuplicateBound,
    DuplicateReceipt,
    ExtraBound,
    MissingBound,
    ExtraReceipt,
    MissingReceipt,
    BlockedReceipt,
}

pub fn reconcile_executions(
    required: &RequiredBag,
    bound: &BoundBag,
    receipts: &ReceiptBag,
) -> Result<(), ExactError> {
    let mut required_keys = BTreeSet::new();
    for key in &required.keys {
        if !required_keys.insert(key) {
            return Err(ExactError::DuplicateRequired);
        }
    }
    let mut bound_keys = BTreeSet::new();
    for plan in &bound.plans {
        if !bound_keys.insert(&plan.key) {
            return Err(ExactError::DuplicateBound);
        }
    }
    let mut receipt_keys = BTreeSet::new();
    for receipt in receipts {
        if !receipt_keys.insert(receipt.key()) {
            return Err(ExactError::DuplicateReceipt);
        }
    }
    if receipt_keys
        .iter()
        .any(|key| required.port_blocked.contains_key(*key) || bound.blocked.contains_key(*key))
    {
        return Err(ExactError::BlockedReceipt);
    }
    if !bound_keys.is_subset(&required_keys) {
        return Err(ExactError::ExtraBound);
    }
    if bound_keys != required_keys {
        return Err(ExactError::MissingBound);
    }
    if !receipt_keys.is_subset(&required_keys) {
        return Err(ExactError::ExtraReceipt);
    }
    if receipt_keys != required_keys {
        return Err(ExactError::MissingReceipt);
    }
    Ok(())
}
