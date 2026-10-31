use super::super::super::{regions, source};
use super::*;

fn records() -> Vec<contracts::Contract> {
    contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap()
}

fn evidence(name: &str, text: &str) {
    let directory = source::workspace().join(".temp/brief-543/falsifiers");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join(format!("{name}.txt")), text).unwrap();
    println!("RULE543_CONTROL {name}: red=reject green=clean");
}

#[test]
fn local_rule_contract_controls() {
    let source = source::production();
    let index = Interfaces::shared_production();
    let records = records();
    let marks = Marks::contracts(records.clone()).unwrap();
    let c02 = resolve::function(
        &source,
        &format!("{}::evaluate_runtime_option_expr", regions::SESSION),
    )
    .unwrap();
    let green = check(&index, c02, &marks);
    assert!(green.rejected.is_empty(), "{}", green.text());
    for record in records
        .iter()
        .filter(|r| r.id.starts_with('R') || ["B04", "B10", "B11", "B12"].contains(&r.id.as_str()))
    {
        assert!(
            marks.lookup(&index, &record.target).is_ok(),
            "{} {:?}",
            record.id,
            marks.lookup(&index, &record.target)
        );
        for pending in [false, true] {
            let mut changed = records.clone();
            let changed_record = changed.iter_mut().find(|r| r.id == record.id).unwrap();
            if pending {
                changed_record.status = "Pending".into();
            } else {
                changed_record.fingerprint.push('0');
            }
            let red = check(&index, c02, &Marks::contracts(changed).unwrap());
            assert!(
                !red.rejected.is_empty(),
                "dependency {} escaped expiry",
                record.id
            );
            evidence(
                &format!(
                    "{}-{}",
                    record.id,
                    if pending { "pending" } else { "stale" }
                ),
                &format!("RED\n{}GREEN 0\n", red.text()),
            );
        }
    }
    // A receipt never follows from the return signature or a known clone type.
    let mut raw = c02.clone();
    raw.body = block(
        "{
        let raw = expr.clone();
        crate::runtime::accounting_mode::rule::clone_result(self, expr)
    }",
    );
    let red = check(&index, &raw, &marks);
    assert!(
        red.rejected.iter().any(|d| d.kind == Kind::RET001),
        "{}",
        red.text()
    );
    evidence(
        "raw-clone-before-adapter",
        &format!("RED\n{}GREEN 0\n", red.text()),
    );
    raw.body = block(
        "{
        let result = crate::runtime::accounting_mode::rule::clone_result(self, expr);
        expr.clone();
        crate::runtime::accounting_mode::rule::clone_result(self, expr)
    }",
    );
    let red = check(&index, &raw, &marks);
    assert!(
        red.rejected.iter().any(|d| d.kind == Kind::RET001),
        "{}",
        red.text()
    );
    evidence(
        "outgoing-result-is-unleased",
        &format!("RED\n{}GREEN 0\n", red.text()),
    );
    for target in ["B10", "B11", "B12"] {
        let red = caller_domain(&index, c02, &contracts::target(target)).unwrap_err();
        evidence(
            &format!("bare-{target}"),
            &format!("RED {red}\nGREEN guarded private domain\n"),
        );
    }
    for id in extra_targets()
        .into_values()
        .filter(|s| !s.contains("::<") && !s.ends_with("::resume"))
    {
        let name = id.rsplit("::").next().unwrap();
        let mut changed = source.clone();
        let f = changed.functions.get_mut(&id).unwrap().first_mut().unwrap();
        f.body = block("{ panic!(\"boundary removed\") }");
        assert!(validate(&changed).values().all(Result::is_err));
        evidence(
            &format!("body-{name}"),
            "RED indexed boundary changed\nGREEN exact declaration\n",
        );
    }
    let mut missing = records.clone();
    missing.retain(|r| r.id != "R03");
    assert!(
        Marks::contracts(missing)
            .unwrap()
            .lookup(&index, &format!("{RULE}::evaluate_rule"))
            .is_err()
    );
    evidence(
        "missing-helper-record",
        "RED missing R03 dependency\nGREEN complete finite records\n",
    );
    let mut schema = source.clone();
    schema.types.get_mut(FROZEN).unwrap()[0].fields[0].name = "untraced".into();
    assert!(validate(&schema).values().all(Result::is_err));
    evidence(
        "changed-frozen-schema",
        "RED changed frozen field\nGREEN pinned field schema\n",
    );
    let mut literal = source.clone();
    let builder = literal
        .functions
        .get_mut(&format!("{RULE}::finish_rule"))
        .unwrap()
        .first_mut()
        .unwrap();
    builder.body = builder.body.replace("\"Rule\"", "\"List\"");
    assert!(validate(&literal).values().all(Result::is_err));
    evidence(
        "wrong-head-literal",
        "RED non-Rule head\nGREEN literal Rule\n",
    );
}

#[test]
fn local_rule_macro_definition_arms_hygiene_and_raw_frame_controls() {
    let source = source::production();
    let index = Interfaces::shared_production();
    assert!(
        index.resume_definition.is_ok(),
        "{:?}",
        index.resume_definition
    );
    let c02 = resolve::function(
        &source,
        &format!("{}::evaluate_runtime_option_expr", regions::SESSION),
    )
    .unwrap();
    let original: syn::Macro = syn::parse_quote!(crate::resume_or_suspend!(
        self, self.evaluate(args[1].clone()),
        ok(value) => Ok(value),
        suspend => RuntimeOptionSuspensionFrame { name: args[0].clone(), },
    ));
    let full = resume_macro::adapt(&source, c02, &original, &index.resume_definition).unwrap();
    let expected: syn::Expr = syn::parse_quote!({
        match self.evaluate(args[1].clone()) {
            Ok(value) => Ok(value),
            Err(crate::kernel_api::EvalError::InputRequired { request }) => {
                self.insert_suspension_frame(RuntimeOptionSuspensionFrame {
                    name: args[0].clone(),
                });
                Err(crate::kernel_api::EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    });
    assert_eq!(tokens(&full.expansion()), tokens(&expected));
    assert!(resume_macro::guarded_insert(&index, c02, &full).is_err());
    evidence(
        "macro-original-raw-frame",
        "GREEN exact three branches; result once; lazy frame; actual error\nRED B12 raw frame domain\n",
    );
    let guarded =
        protocol::signature(&source, &format!("{}::suspend_error", protocol::CONTEXT)).unwrap();
    struct Macros(Vec<syn::Macro>);
    impl<'a> Visit<'a> for Macros {
        fn visit_macro(&mut self, mac: &'a syn::Macro) {
            if tokens(&mac.path) == "crate :: resume_or_suspend" {
                self.0.push(mac.clone());
            }
        }
    }
    let mut macros = Macros(vec![]);
    macros.visit_block(&guarded.body());
    let [insert] = macros.0.as_slice() else {
        panic!("one insert arm")
    };
    let arm = resume_macro::adapt(&source, guarded, insert, &index.resume_definition).unwrap();
    resume_macro::guarded_insert(&index, guarded, &arm).unwrap();
    let raw_insert: syn::Macro = syn::parse_quote!(
        crate::resume_or_suspend!(insert self.session, RuntimeOptionSuspensionFrame { name: value })
    );
    let raw_arm =
        resume_macro::adapt(&source, guarded, &raw_insert, &index.resume_definition).unwrap();
    assert!(resume_macro::guarded_insert(&index, guarded, &raw_arm).is_err());
    evidence(
        "macro-raw-insert",
        "RED raw frame at same insertion callee\nGREEN private charged insertion\n",
    );
    for (name, mutated) in [
        ("macro-definition", "Err(error) => Err(error),"),
        (
            "macro-insertion",
            "$session.insert_suspension_frame($frame);",
        ),
    ] {
        let mut changed = source.clone();
        let file = changed
            .files
            .get_mut("crates/rum-runtime/src/runtime/suspension/core.rs")
            .unwrap();
        *file = file.replace(mutated, "Err(error) => panic!(),");
        assert!(resume_macro::definition(&changed).is_err());
        evidence(
            name,
            "RED changed definition/arm\nGREEN exact canonical definition\n",
        );
    }
    let mut alias = original.clone();
    alias.path = syn::parse_quote!(resume_or_suspend);
    assert!(resume_macro::adapt(&source, c02, &alias, &index.resume_definition).is_err());
    let mut shadow = c02.clone();
    shadow.body = block(
        "{
        let Ok = 1;
    }",
    );
    assert!(resume_macro::adapt(&source, &shadow, &original, &index.resume_definition).is_err());
    let malformed: syn::Macro = syn::parse_quote!(
        crate::resume_or_suspend!(self, result, ok((a,b)) => Ok(a), suspend => frame)
    );
    assert!(resume_macro::adapt(&source, c02, &malformed, &index.resume_definition).is_err());
    evidence(
        "macro-hygiene",
        "RED alias, local Ok and unsupported pattern\nGREEN qualified exact arms\n",
    );
}
