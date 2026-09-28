//! Watched rejection/restoration at the production certificate boundary.
use super::super::source;
use super::*;

fn setup() -> (Source, Interfaces, Marks) {
    let source = promotion::population(source::production());
    let index = Interfaces::new(&source);
    let marks = promotion::earn(&source, &index);
    (source, index, marks)
}

fn witness(label: &str, red: &str) {
    assert!(!red.is_empty());
    let path = source::workspace().join(".temp/brief-546/falsifiers");
    std::fs::create_dir_all(&path).unwrap();
    let text = format!("{label}: RED {red}\n{label}: GREEN restored original evidence\n");
    std::fs::write(
        path.join(format!("{label}-hash-{}.txt", cfg!(feature = "hash"))),
        &text,
    )
    .unwrap();
    print!("{text}");
}

#[test]
fn production_marks_require_current_local_lint_and_direct_contracts() {
    let (source, index, marks) = setup();
    let records = contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap();
    for (label, id) in certificates::roots() {
        let body = resolve::function(&source, &id).unwrap();
        let dependencies: BTreeSet<_> = marks
            .production_records(&index)
            .unwrap()
            .into_iter()
            .filter(|(i, _)| i.body == id)
            .flat_map(|(_, d)| d.keys().cloned())
            .collect();
        for dependency in dependencies {
            let record = records.iter().find(|r| r.target == dependency).unwrap();
            for mutation in ["missing", "stale", "Pending"] {
                let mut changed = records.clone();
                if mutation == "missing" {
                    changed.retain(|r| r.target != dependency);
                } else {
                    let row = changed.iter_mut().find(|r| r.target == dependency).unwrap();
                    if mutation == "stale" {
                        row.fingerprint.push('0');
                    } else {
                        row.status = "Pending".into();
                    }
                }
                let mut bad = Marks::contracts(changed).unwrap();
                let revoked = bad.with_covered(&marks);
                for (instance, _) in marks
                    .production_records(&index)
                    .unwrap()
                    .into_iter()
                    .filter(|(i, _)| i.body == id)
                {
                    assert!(
                        revoked.lookup(&index, &instance.key).is_err(),
                        "previously earned {} stayed covered after {mutation} {}",
                        instance.key,
                        record.id
                    );
                }
                let red = bad.certify(&index, body).unwrap_err();
                assert!(!red.rejected.is_empty(), "{label} {mutation} {}", record.id);
                let text = red.text();
                match mutation {
                    "missing" => assert!(
                        text.contains(&dependency)
                            && (text.contains("callee has no covered mark")
                                || text.contains("missing rule boundary dependency")),
                        "{text}"
                    ),
                    "stale" => assert!(
                        text.contains(&format!("stale contract fingerprint {}", record.id)),
                        "{text}"
                    ),
                    _ => assert!(
                        text.contains(&format!("Pending contract {} is not a mark", record.id)),
                        "{text}"
                    ),
                }
                assert_eq!(
                    bad.count(),
                    0,
                    "failed certification cannot publish an instance"
                );
                let mut restored = Marks::contracts(records.clone()).unwrap();
                restored.certify(&index, body).unwrap();
                witness(&format!("{label}-{}-{mutation}", record.id), &red.text());
            }
        }
    }
    assert!(
        marks
            .lookup(&index, &option_boundaries::c01())
            .unwrap_err()
            .contains("no covered mark")
    );
    let generic = resolve::function(&source, &option_boundaries::c01()).unwrap();
    let all_r = check_instance(&index, generic, &marks);
    assert!(
        !all_r.rejected.is_empty(),
        "finite instances must not certify all R"
    );
    assert!(
        certificates::instances(&index, generic)
            .unwrap()
            .iter()
            .all(|(i, _)| i.reentry == certificates::DESCRIPTOR)
    );
    witness("all-R", &all_r.text());
}

#[test]
fn production_body_domain_and_build_expiry_and_raw_local() {
    let (source, mut index, marks) = setup();
    for (label, id) in certificates::roots() {
        let mut changed = source.clone();
        changed.functions.get_mut(&id).unwrap()[0]
            .body
            .stmts
            .insert(
                0,
                syn::parse_quote!(let raw = std::vec::Vec::<rum_core::Expr>::new();),
            );
        let other = Interfaces::new(&changed);
        let raw = resolve::function(&changed, &id).unwrap();
        let mut attempt = Marks::contracts(
            contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap(),
        )
        .unwrap();
        let red = attempt.certify(&other, raw).unwrap_err();
        assert!(
            red.rejected
                .iter()
                .any(|d| d.kind == Kind::RET001 && d.reason.contains("is live across call")),
            "{}",
            red.text()
        );
        assert_eq!(attempt.count(), 0);
        let expired = marks
            .production_records(&other)
            .err()
            .expect("body fingerprint must expire");
        assert!(expired.contains("body fingerprint"));
        attempt
            .certify(&index, resolve::function(&source, &id).unwrap())
            .unwrap();
        witness(&format!("{label}-raw-across-guest"), &red.text());
        witness(&format!("{label}-expired-body"), &expired);
    }
    let mut changed = source.clone();
    changed
        .files
        .get_mut(option_domain::DECLARATION)
        .unwrap()
        .push_str("\n// domain fingerprint falsifier\n");
    let other = Interfaces::new(&changed);
    let red = marks
        .production_records(&other)
        .err()
        .expect("domain source must expire");
    assert!(red.contains("domain fingerprint"), "{red}");
    promotion::expected(&index, &marks).unwrap();
    witness("expired-domain", &red);
    let build = index.build.clone();
    index.build.push_str(";changed-build");
    let red = marks.production_records(&index).err().unwrap();
    assert!(red.contains("build"));
    index.build = build;
    promotion::expected(&index, &marks).unwrap();
    witness("expired-build", &red);
}

fn changed_cell(inventory: &str, row: usize, field: usize, value: &str) -> String {
    let mut rows: Vec<Vec<String>> = inventory
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect();
    rows[row][field] = value.into();
    rows.into_iter().map(|r| r.join("\t") + "\n").collect()
}

#[test]
fn production_inventory_rejects_removed_redirected_stale_and_omitted_evidence() {
    let (source, index, marks) = setup();
    let active = if cfg!(feature = "hash") { 4 } else { 5 };
    let rows: Vec<_> = promotion::INVENTORY
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert_eq!(rows.len(), 2);
    let check = |text: &str| {
        promotion::reconcile(
            &source,
            &index,
            &marks,
            text,
            promotion::OBLIGATIONS,
            promotion::CONSUMERS,
        )
    };
    check(promotion::INVENTORY).unwrap();
    let mut mutations = vec![("absent".to_string(), String::new())];
    for (n, row) in rows.iter().enumerate() {
        mutations.push((format!("removed-C0{}", n + 1), rows[1 - n].to_string()));
        mutations.push((
            format!("redirected-C0{}", n + 1),
            changed_cell(promotion::INVENTORY, n, 2, "outside::caller"),
        ));
        mutations.push((
            format!("expired-C0{}", n + 1),
            changed_cell(
                promotion::INVENTORY,
                n,
                active,
                &(row.split('\t').nth(active).unwrap().to_string() + "stale"),
            ),
        ));
        mutations.push((
            format!("exclusions-C0{}", n + 1),
            changed_cell(promotion::INVENTORY, n, 8, "none"),
        ));
        mutations.push((
            format!("absent-other-build-C0{}", n + 1),
            changed_cell(promotion::INVENTORY, n, 9 - active, "UNRECORDED"),
        ));
        let instances: Vec<_> = row.split('\t').nth(active).unwrap().split(" || ").collect();
        for omitted in 0..instances.len() {
            let remaining = instances
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != omitted)
                .map(|(_, s)| *s)
                .collect::<Vec<_>>()
                .join(" || ");
            mutations.push((
                format!("omitted-C0{}-instance-{omitted}", n + 1),
                changed_cell(promotion::INVENTORY, n, active, &remaining),
            ));
        }
    }
    for (label, text) in mutations {
        let red = check(&text).unwrap_err();
        check(promotion::INVENTORY).unwrap();
        witness(&label, &red);
    }
    let absent = Marks::contracts(
        contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap(),
    )
    .unwrap();
    let red = promotion::expected(&index, &absent).unwrap_err();
    assert!(red.contains("missing"));
    promotion::expected(&index, &marks).unwrap();
    witness("unearned-inventory", &red);
}

#[test]
fn production_promotion_cannot_launder_consumers_or_runtime_domain() {
    let (source, index, marks) = setup();
    for row in promotion::CONSUMERS
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let id = row.split('\t').next().unwrap();
        let bad = promotion::CONSUMERS.replace(
            &format!("{id}\tUnresolvedConsumer"),
            &format!("{id}\tCovered"),
        );
        let red = promotion::reconcile(
            &source,
            &index,
            &marks,
            promotion::INVENTORY,
            promotion::OBLIGATIONS,
            &bad,
        )
        .unwrap_err();
        assert!(red.contains("claimed covered"));
        promotion::reconcile(
            &source,
            &index,
            &marks,
            promotion::INVENTORY,
            promotion::OBLIGATIONS,
            promotion::CONSUMERS,
        )
        .unwrap();
        witness(id, &red);
    }
    let bad = promotion::OBLIGATIONS.replacen("\tPending\t", "\tLocalCovered\t", 1);
    let red = promotion::reconcile(
        &source,
        &index,
        &marks,
        promotion::INVENTORY,
        &bad,
        promotion::CONSUMERS,
    )
    .unwrap_err();
    assert!(red.contains("caller obligation"));
    promotion::reconcile(
        &source,
        &index,
        &marks,
        promotion::INVENTORY,
        promotion::OBLIGATIONS,
        promotion::CONSUMERS,
    )
    .unwrap();
    witness("caller-obligation", &red);
    let mut session = rum_runtime::Session::new();
    crate::install_core(&mut session);
    let keys = option_domain::keys(index.option_domain.as_ref().unwrap());
    session.install_option_reentry_domain(&[]);
    let red = option_domain::reconcile(&source, &session, &keys).unwrap_err();
    assert!(red.contains("runtime"));
    crate::install_core(&mut session);
    option_domain::reconcile(&source, &session, &keys).unwrap();
    witness("runtime-certified-set", &red);
}
