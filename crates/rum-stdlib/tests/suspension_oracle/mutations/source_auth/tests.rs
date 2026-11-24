use super::*;

#[test]
fn cm_mutation_source_export() {
    let Ok(output) = std::env::var("RUM_CM740_SCHEMA_OUT") else {
        return;
    };
    let workspace = crate::oracle::coverage::source::workspace();
    let output = PathBuf::from(output);
    assert!(admitted_scratch(&workspace, &output));
    assert!(!output.exists(), "preserve every exported schema");
    let mut manifest = String::from("arm\tfile\tpreimage\tpostimage\n");
    for (arm, enabled) in [("disabled", false), ("armed", true)] {
        for (file, pin) in schema::pins() {
            let before = std::fs::read(workspace.join(file)).unwrap();
            assert_eq!(digest(&before), pin);
            let after = schema::file_postimage(file, &before, enabled).unwrap();
            let path = output.join(arm).join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &after).unwrap();
            manifest.push_str(&format!("{arm}\t{file}\t{pin}\t{}\n", digest(&after)));
        }
    }
    std::fs::write(output.join("overlays.tsv"), manifest).unwrap();
    println!(
        "CM4 SOURCE SCHEMA EXPORTED own_rows=151 overlays_per_arm={} new_live_runtime_accessors=0 path={}",
        schema::pins().len(),
        output.display()
    );
}

fn addition_bytes(file: &str, len: usize, pin: &str) -> Vec<u8> {
    let mut candidates = Vec::new();
    if let Some(root) = std::env::var_os("RUM_CM740_ADDITIONS") {
        candidates.push(PathBuf::from(root).join(file));
    }
    candidates.push(crate::oracle::coverage::source::workspace().join(file));
    for path in &candidates {
        if let Ok(bytes) = std::fs::read(path) {
            assert_eq!(bytes.len(), len, "{file}");
            assert_eq!(digest(&bytes), pin, "{file}");
            return bytes;
        }
    }
    panic!("addition bytes unavailable: {file}");
}

fn control<A: Arm>(baseline: &Snapshot) {
    // Validator controls only: these in-memory values are never build or catch receipts.
    let additions = schema::additions().expect("additions");
    let mut before = baseline.clone();
    for file in additions.keys() {
        before.files.remove(*file);
    }
    let mut copied = before.clone();
    for (file, (pin, len)) in &additions {
        copied
            .files
            .insert((*file).to_string(), addition_bytes(file, *len, pin));
    }
    for file in schema::pins().keys() {
        copied.files.insert(
            (*file).into(),
            schema::file_postimage(file, &before.files[*file], A::ENABLED).unwrap(),
        );
    }
    let receipt = Receipt {
        fields: [
            ("arm", A::NAME),
            ("baseline_tree", &before.tree),
            ("binary", "validator-control"),
            ("codegen_units", "1"),
            ("configuration", "default"),
            ("profile", "release"),
            ("tree", &copied.tree),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect(),
        overlays: schema::pins()
            .into_iter()
            .map(|(file, pin)| (file.into(), (pin.into(), digest(&copied.files[file]))))
            .collect(),
    };
    assert_eq!(
        verify::<A>(&before, &copied, "validator-control", &receipt),
        Ok(())
    );
    for (name, expected) in [
        ("unlisted-changed-file", AuthError::UnlistedFile),
        ("wrong-preimage", AuthError::Preimage),
        ("wrong-postimage", AuthError::Postimage),
        ("wrong-binary", AuthError::Binary),
        ("missing-overlay", AuthError::OverlaySet),
        ("mutated-inventory", AuthError::Inventory),
        ("disabled-armed-swap", AuthError::WrongArm),
        ("wrong-tree", AuthError::Tree),
        ("wrong-source-seal", AuthError::BaselineSeal),
    ] {
        let mut altered = copied.clone();
        let mut proposal = receipt.clone();
        let mut before = before.clone();
        match name {
            "unlisted-changed-file" => {
                altered.files.get_mut("README.md").unwrap().push(b'\n');
            }
            "wrong-preimage" => {
                proposal.overlays.get_mut(schema::SOURCE).unwrap().0 = "wrong".into();
            }
            "wrong-postimage" => {
                proposal.overlays.get_mut(schema::SOURCE).unwrap().1 = "wrong".into();
            }
            "wrong-binary" => {
                proposal.fields.insert("binary".into(), "wrong".into());
            }
            "missing-overlay" => {
                proposal.overlays.clear();
            }
            "mutated-inventory" => {
                altered.files.get_mut("crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/bindings.tsv").unwrap().push(b'\n');
            }
            "disabled-armed-swap" => {
                proposal.fields.insert(
                    "arm".into(),
                    if A::ENABLED { "disabled" } else { "armed" }.into(),
                );
            }
            "wrong-tree" => {
                proposal.fields.insert("tree".into(), "wrong".into());
            }
            "wrong-source-seal" => {
                before.files.get_mut(schema::SOURCE).unwrap().push(b'\n');
            }
            _ => unreachable!(),
        }
        assert_eq!(
            verify::<A>(&before, &altered, "validator-control", &proposal),
            Err(expected),
            "{name}"
        );
        println!(
            "CM4 SOURCE AUTH NEGATIVE arm={} control={name} predicate={expected:?} semantic_catch=false",
            A::NAME
        );
    }
    // An arm label cannot bless the other arm's actual carrier bytes.
    let mut swapped = copied.clone();
    swapped.files.insert(
        schema::SOURCE.into(),
        schema::postimage(&before.files[schema::SOURCE], !A::ENABLED).unwrap(),
    );
    let mut matching_hash = receipt.clone();
    matching_hash.overlays.get_mut(schema::SOURCE).unwrap().1 =
        digest(&swapped.files[schema::SOURCE]);
    assert_eq!(
        verify::<A>(&before, &swapped, "validator-control", &matching_hash),
        Err(AuthError::Postimage)
    );
    println!(
        "CM4 SOURCE AUTH NEGATIVE arm={} control=carrier-arm-swap predicate=Postimage semantic_catch=false",
        A::NAME
    );
    let sample = *additions.keys().next().unwrap();
    let mut extra = copied.clone();
    extra
        .files
        .insert("unlisted-addition.rs".into(), b"extra".to_vec());
    assert_eq!(
        verify::<A>(&before, &extra, "validator-control", &receipt),
        Err(AuthError::UnlistedFile)
    );
    println!(
        "CM4 SOURCE AUTH NEGATIVE arm={} control=extra-unpinned-file predicate=UnlistedFile semantic_catch=false",
        A::NAME
    );
    let mut missing = copied.clone();
    missing.files.remove(sample);
    assert_eq!(
        verify::<A>(&before, &missing, "validator-control", &receipt),
        Err(AuthError::UnlistedFile)
    );
    println!(
        "CM4 SOURCE AUTH NEGATIVE arm={} control=missing-addition predicate=UnlistedFile semantic_catch=false",
        A::NAME
    );
    let mut wrong = copied.clone();
    wrong.files.get_mut(sample).unwrap()[0] ^= 0xff;
    assert_eq!(
        verify::<A>(&before, &wrong, "validator-control", &receipt),
        Err(AuthError::UnlistedFile)
    );
    println!(
        "CM4 SOURCE AUTH NEGATIVE arm={} control=addition-wrong-digest predicate=UnlistedFile semantic_catch=false",
        A::NAME
    );
    let mut present = before.clone();
    present
        .files
        .insert(sample.to_string(), copied.files[sample].clone());
    assert_eq!(
        verify::<A>(&present, &copied, "validator-control", &receipt),
        Err(AuthError::UnlistedFile)
    );
    println!(
        "CM4 SOURCE AUTH NEGATIVE arm={} control=addition-in-baseline predicate=UnlistedFile semantic_catch=false",
        A::NAME
    );
    let mut listed = receipt.clone();
    listed.overlays.insert(
        sample.to_string(),
        (
            additions[sample].0.to_string(),
            digest(&copied.files[sample]),
        ),
    );
    assert_eq!(
        verify::<A>(&before, &copied, "validator-control", &listed),
        Err(AuthError::OverlaySet)
    );
    println!(
        "CM4 SOURCE AUTH NEGATIVE arm={} control=addition-listed-as-overlay predicate=OverlaySet semantic_catch=false",
        A::NAME
    );
    for (name, text) in [
        ("wrong-header", "file\tsha\tbytes\n"),
        (
            "duplicate",
            "file\tsha256\tbytes\na\t0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\t1\na\t0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\t1\n",
        ),
        ("non-hex", "file\tsha256\tbytes\na\tzz\t1\n"),
        (
            "bad-length",
            "file\tsha256\tbytes\na\t0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\tno\n",
        ),
    ] {
        assert_eq!(
            schema::parse_additions(text),
            Err(AuthError::OverlaySet),
            "{name}"
        );
        println!(
            "CM4 SOURCE AUTH NEGATIVE arm={} control=malformed-additions-{name} predicate=OverlaySet semantic_catch=false",
            A::NAME
        );
    }
    let mut longer = copied.clone();
    longer.files.get_mut(sample).unwrap().push(b'\n');
    assert_eq!(
        verify::<A>(&before, &longer, "validator-control", &receipt),
        Err(AuthError::OverlaySet)
    );
    println!(
        "CM4 SOURCE AUTH NEGATIVE arm={} control=addition-wrong-length predicate=OverlaySet semantic_catch=false",
        A::NAME
    );
    println!(
        "CM4 SOURCE AUTH ADDITIONS PASS arm={} rows={}",
        A::NAME,
        additions.len()
    );
    println!(
        "CM4 SOURCE AUTH VALIDATOR PASS arm={} negative_controls=10 build_receipts=0",
        A::NAME
    );
}

#[test]
fn cm_mutation_source_authentication_controls() {
    let baseline = std::env::var_os("RUM_CM740_BASELINE")
        .map_or_else(crate::oracle::coverage::source::workspace, PathBuf::from);
    let snapshot = Snapshot::read(&baseline).unwrap();
    control::<Disabled>(&snapshot);
    control::<Armed>(&snapshot);
}

#[test]
fn cm_mutation_source_private_types() {
    let workspace = crate::oracle::coverage::source::workspace();
    let scratch = std::env::var_os("RUM_CM740_SCRATCH").map_or_else(
        || workspace.join(".temp/brief-740/type-controls"),
        PathBuf::from,
    );
    let directory = scratch.join(format!(
        "privacy-{}-{}",
        std::process::id(),
        cfg!(feature = "hash")
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let view =
        workspace.join("crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/view.rs");
    let prefix = format!(
        "mod authentication {{ #[path = {view:?}] pub mod view; }}\nuse authentication::view::*;\n"
    );
    for (name, body, expected) in [
        (
            "positive",
            "fn typed(_: &AuthenticatedSource<Disabled>) {}",
            None,
        ),
        (
            "forged",
            "fn forged() -> AuthenticatedSource<Disabled> { AuthenticatedSource { root: Default::default(), originals: Default::default(), postimages: Default::default(), arm: Default::default() } }",
            Some("E0451"),
        ),
        (
            "wrong_arm",
            "fn armed(_: &AuthenticatedSource<Armed>) {} fn misuse(disabled: &AuthenticatedSource<Disabled>) { armed(disabled); }",
            Some("E0308"),
        ),
        (
            "observer_positive",
            "fn observe(_: &MutationObserver<'_>) {}",
            None,
        ),
        (
            "observer_forged",
            "fn forged(source: &AuthenticatedSource<Armed>) -> MutationObserver<'_> { MutationObserver { source, row: \"C10.prepared.profiled_consumer\" } }",
            Some("E0451"),
        ),
        (
            "observer_unauthenticated",
            "fn observe(_: &MutationObserver<'_>) {} fn misuse(source: &AuthenticatedSource<Armed>) { observe(source); }",
            Some("E0308"),
        ),
        (
            "observer_wrong_arm",
            "fn forged(source: &AuthenticatedSource<Disabled>) -> MutationObserver<'_> { MutationObserver { source, row: \"C10.prepared.profiled_consumer\" } }",
            Some("E0308"),
        ),
    ] {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{prefix}{body}\n")).unwrap();
        let output = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "--out-dir",
            ])
            .arg(&directory)
            .arg(&path)
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        std::fs::write(directory.join(format!("{name}.stderr")), &stderr).unwrap();
        match expected {
            None => assert!(output.status.success(), "{stderr}"),
            Some(code) => assert!(
                !output.status.success() && stderr.contains(code),
                "{stderr}"
            ),
        }
        println!(
            "CM4 SOURCE TYPE PASS control={name} expected={expected:?} semantic_catch=false scratch={}",
            directory.display()
        );
    }
}

#[test]
fn cm_mutation_source_route_postimage() {
    // This tests the use-time guard only, not the construction of a build receipt.
    let workspace = crate::oracle::coverage::source::workspace();
    let root = workspace.join(format!(
        ".temp/brief-740/route-controls/{}-{}",
        std::process::id(),
        cfg!(feature = "hash")
    ));
    std::fs::create_dir_all(&root).unwrap();
    let file = "route.rs";
    let original = b"original route".to_vec();
    let postimage = b"authenticated copied route";
    std::fs::write(root.join(file), postimage).unwrap();
    let view = AuthenticatedSource::<Disabled> {
        root: root.clone(),
        originals: [(file.into(), original.clone())].into(),
        postimages: [(file.into(), digest(postimage))].into(),
        arm: PhantomData,
    };
    assert_eq!(view.source(file), Ok(original));
    std::fs::write(root.join(file), b"changed after authentication").unwrap();
    assert_eq!(
        view.source(file),
        Err(bindings::BindingError::SourceIdentity)
    );
    assert_eq!(
        view.source("unlisted.rs"),
        Err(bindings::BindingError::SourceIdentity)
    );
    println!(
        "CM4 SOURCE ROUTE PASS original_only_after_postimage_match=true changed_postimage=SourceIdentity unlisted_path=SourceIdentity semantic_catch=false scratch={}",
        root.display()
    );
}

fn anchor_falsifier(baseline: &std::path::Path, planted: &str) {
    let workspace = crate::oracle::coverage::source::workspace();
    let seals = crate::oracle::obligations::bindings::provenance().unwrap();
    let class_bytes = std::fs::read(workspace.join(planted)).unwrap();
    let seal = seals
        .iter()
        .find(|row| row[0] == "canonical-source" && row[2] == planted)
        .unwrap();
    assert_ne!(
        digest(&class_bytes),
        seal[3],
        "class-form bytes must differ from the baseline seal"
    );
    let mut originals = BTreeMap::new();
    let mut postimages = BTreeMap::new();
    for row in seals.iter().filter(|row| row[0] == "canonical-source") {
        let file = row[2];
        let workspace_bytes = std::fs::read(workspace.join(file)).unwrap();
        postimages.insert(file.to_string(), digest(&workspace_bytes));
        if file == planted {
            originals.insert(file.to_string(), class_bytes.clone());
        } else {
            let prior = std::fs::read(baseline.join(file)).unwrap();
            assert_eq!(digest(&prior), row[3], "{file}");
            originals.insert(file.to_string(), prior);
        }
    }
    let view = AuthenticatedSource::<Disabled> {
        root: workspace,
        originals,
        postimages,
        arm: PhantomData,
    };
    assert_eq!(
        crate::oracle::fixtures::obligations::guest::programs_with_mutation(&view).unwrap_err(),
        bindings::BindingError::SourceIdentity
    );
    println!(
        "CM4 COPIED SOURCE ANCHOR FALSIFIER PASS predicate=SourceIdentity baseline_refuses_class_bytes=true path={planted}"
    );
}

#[test]
fn cm_mutation_source_copied_admission() {
    let Some((baseline, receipt_path)) = environment() else {
        return;
    };
    let receipt = Receipt::read(&receipt_path).unwrap();
    let arm = receipt.fields["arm"].as_str();
    let programs = match arm {
        "disabled" => crate::oracle::fixtures::obligations::guest::programs_with_mutation(
            &authenticate::<Disabled>(&baseline, &receipt_path).unwrap(),
        ),
        "armed" => crate::oracle::fixtures::obligations::guest::programs_with_mutation(
            &authenticate::<Armed>(&baseline, &receipt_path).unwrap(),
        ),
        _ => panic!("unrecognized arm"),
    }
    .unwrap();
    assert_eq!(programs.len(), 9060);
    assert_eq!(
        programs.values().filter(|p| p.key.row() == "P0543").count(),
        55
    );
    match arm {
        "disabled" => {
            assert_eq!(
                crate::oracle::fixtures::obligations::guest::programs().unwrap_err(),
                bindings::BindingError::SourceIdentity
            );
            println!(
                "CM4 COPIED SOURCE ADMISSION PASS arm=disabled programs=9060 P0543=55 strict_loader=SourceIdentity"
            );
        }
        "armed" => {
            let loaded = crate::oracle::fixtures::obligations::guest::programs().unwrap();
            assert_eq!(loaded.len(), 9060);
            println!(
                "CM4 COPIED SOURCE ADMISSION PASS arm=armed programs=9060 P0543=55 strict_loader=class_form_ok"
            );
        }
        _ => panic!("unrecognized arm"),
    }
    anchor_falsifier(&baseline, "crates/rum-host/src/session.rs");
}

#[test]
fn cm_mutation_observer_copied_capability() {
    let Some((baseline, receipt_path)) = environment() else {
        return;
    };
    let receipt = Receipt::read(&receipt_path).unwrap();
    if receipt.fields["arm"] == "disabled" {
        assert!(matches!(
            authenticate::<Armed>(&baseline, &receipt_path),
            Err(AuthError::WrongArm)
        ));
        println!(
            "CM4 OBSERVER COPIED NEGATIVE control=disabled-armed-swap predicate=WrongArm observation_supplied=false"
        );
        return;
    }
    let source = authenticate::<Armed>(&baseline, &receipt_path).unwrap();
    let row = std::env::var("RUM_CM740_ROW").unwrap_or_else(|_| observer::ROW.into());
    let permission = source.observer(&row).unwrap();
    let programs =
        crate::oracle::fixtures::obligations::guest::programs_with_mutation(&source).unwrap();
    let original = programs.values().find(|p| p.key.row() == row).unwrap();
    permission.check(original).unwrap();
    let mut wrong = original.clone();
    wrong.key.0[0] = "P0543".into();
    assert_eq!(permission.check(&wrong), Err(AuthError::ObserverRow));
    assert_eq!(
        crate::oracle::fixtures::obligations::guest::run_with_mutation_observer(
            &wrong,
            false,
            &permission
        ),
        Err(crate::oracle::fixtures::obligations::guest::RunFailure::Program)
    );
    let mut changed = original.clone();
    changed.source.push(' ');
    assert_eq!(permission.check(&changed), Err(AuthError::ObserverProgram));
    println!(
        "CM4 OBSERVER COPIED CAPABILITY PASS row={row} wrong_row=ObserverRow changed_program=ObserverProgram observation_supplied=false"
    );
}

#[test]
fn cm_mutation_source_gate_result() {
    let Some((baseline, receipt_path)) = environment() else {
        return;
    };
    let Ok(which) = std::env::var("RUM_CM740_GATE") else {
        return;
    };
    let result = match which.as_str() {
        "armed" => authenticate::<Armed>(&baseline, &receipt_path).map(|_| ()),
        "disabled" => authenticate::<Disabled>(&baseline, &receipt_path).map(|_| ()),
        other => panic!("unrecognized gate {other}"),
    };
    match &result {
        Ok(()) => println!("CM4 SOURCE GATE PASS auth={which}"),
        Err(error) => println!(
            "CM4 SOURCE GATE NEGATIVE auth={which} predicate={error:?} semantic_catch=false"
        ),
    }
    if let Ok(expect) = std::env::var("RUM_CM740_EXPECT") {
        let got = match &result {
            Ok(()) => "Ok".to_string(),
            Err(error) => format!("{error:?}"),
        };
        assert_eq!(got, expect);
    }
}
