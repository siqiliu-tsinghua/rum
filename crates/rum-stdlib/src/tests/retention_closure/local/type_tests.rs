use super::super::source;
use super::*;

#[test]
fn finite_profile_frontier_discovery() {
    use super::super::{methods, regions};
    let source = source::production();
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let frontier = regions::frontier(
        &source,
        &[
            format!("{}::merged_option_exprs_for_symbol", regions::SESSION),
            format!("{}::evaluate_runtime_option_expr", regions::SESSION),
        ],
    );
    let target = format!("{}::abandon_frames", regions::SESSION);
    let instances: BTreeSet<_> = frontier
        .methods
        .values()
        .filter(|s| s.function.id == target)
        .map(|s| methods::key(&s.function))
        .collect();
    assert_eq!(
        instances.len(),
        2,
        "both finite instances must enter the frontier"
    );
    assert!(frontier.bodies.contains(&target));
    let mut kinds = BTreeMap::new();
    let mut rows = String::new();
    for row in &frontier.rejected {
        *kinds.entry(row.kind).or_insert(0usize) += 1;
        rows.push_str(&format!("{row}\n"));
    }
    let directory = source::workspace().join(".temp/brief-626/frontier");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("rejected.tsv"), rows).unwrap();
    std::fs::write(
        directory.join("bodies.txt"),
        frontier.bodies.into_iter().collect::<Vec<_>>().join("\n"),
    )
    .unwrap();
    for instance in instances {
        println!("LP02_FRONTIER_INSTANCE {}", instance.display());
    }
    println!(
        "LP02_FRONTIER total={} kinds={kinds:?}",
        frontier.rejected.len()
    );
}

#[test]
fn finite_profile_modes_are_closed_and_selected_at_both_calls() {
    use super::super::{methods, regions};
    let source = source::production();
    let profile = &types::MODE_DOMAINS[1];
    let target = format!("{}::abandon_frames", regions::SESSION);
    let frame = resolve::function(&source, &target).unwrap();
    for id in profile.implementors {
        let mode = ExactTy::Nominal((*id).into(), vec![]);
        let instance = types::with_mode(&source, frame, Some(&mode)).unwrap();
        assert_eq!(instance.substitutions["M"], mode);
        assert_eq!(
            types::exact_output(&source, &instance).unwrap(),
            ExactTy::Unit
        );
        let guard = if id.ends_with("::Ordinary") {
            ExactTy::Unit
        } else {
            ExactTy::Nominal(
                "std::option::Option".into(),
                vec![ExactTy::Nominal(
                    "rum_runtime::runtime::profiling::logical::context::ContextGuard".into(),
                    vec![],
                )],
            )
        };
        for syntax in [
            syn::parse_quote!(M::Guard),
            syn::parse_quote!(<M as crate::runtime::suspension::core::profile_mode::Mode>::Guard),
        ] {
            assert_eq!(
                types::in_function(&source, &instance, &syntax).unwrap(),
                guard
            );
        }
        assert_eq!(
            types::in_function(&source, &instance, &syn::parse_quote!(&M::Guard)).unwrap(),
            ExactTy::Reference(Box::new(guard), false)
        );
        assert!(types::in_function(&source, &instance, &syn::parse_quote!(M::Missing)).is_err());
        assert!(types::in_function(&source, &instance, &syn::parse_quote!(M::Guard<u8>)).is_err());
        let mut missing = source.clone();
        for (module, item, _, _) in &mut missing.trait_impls {
            let mut parsed = ast::<syn::ItemImpl>(item);
            if parsed.trait_.as_ref().is_some_and(|(_, path, _)| {
                resolve::path(&source, module, path).ok().as_deref() == Some(profile.trait_path)
            }) {
                parsed
                    .items
                    .retain(|item| !matches!(item, syn::ImplItem::Type(t) if t.ident == "Guard"));
            }
            *item = tokens(&parsed);
        }
        assert!(types::in_function(&missing, &instance, &syn::parse_quote!(M::Guard)).is_err());
    }
    assert!(
        types::with_mode(
            &source,
            frame,
            Some(&ExactTy::Nominal(types::MODES[0].into(), vec![]))
        )
        .is_err()
    );
    let caller = resolve::function(
        &source,
        &format!("{}::drain_abandonment_worklist", regions::SESSION),
    )
    .unwrap();
    let profiled_caller = resolve::function(
        &source,
        &format!("{}::abandon_frames_profiled", regions::SESSION),
    )
    .unwrap();
    let callers = [caller, profiled_caller];
    let selections: Vec<_> = callers
        .into_iter()
        .flat_map(|caller| {
            methods::claims(&source, caller)
                .into_values()
                .collect::<Vec<_>>()
        })
        .filter_map(Result::ok)
        .filter(|s| s.function.id == target)
        .collect();
    assert_eq!(
        selections.len(),
        2,
        "both explicit mode calls must be traversed"
    );
    let modes: BTreeSet<_> = selections
        .iter()
        .map(|s| s.function.substitutions["M"].clone())
        .collect();
    assert_eq!(
        modes,
        profile
            .implementors
            .iter()
            .map(|id| ExactTy::Nominal((*id).into(), vec![]))
            .collect()
    );

    for domain in types::MODE_DOMAINS {
        domain.check(&source).unwrap();
        let members: Vec<_> = source
            .trait_impls
            .iter()
            .enumerate()
            .filter(|(_, (module, item, _, _))| {
                ast::<syn::ItemImpl>(item)
                    .trait_
                    .as_ref()
                    .is_some_and(|(_, path, _)| {
                        resolve::path(&source, module, path).ok().as_deref()
                            == Some(domain.trait_path)
                    })
            })
            .map(|(i, _)| i)
            .collect();
        assert_eq!(members.len(), 2);
        for &index in &members {
            let mut missing = source.clone();
            missing.trait_impls.remove(index);
            assert!(
                domain.check(&missing).is_err(),
                "either missing impl closes the domain"
            );
            if domain.trait_path == profile.trait_path {
                assert!(
                    callers.into_iter().all(|caller| {
                        methods::claims(&missing, caller)
                            .into_values()
                            .filter_map(Result::ok)
                            .all(|s| s.function.id != target)
                    }),
                    "missing impl must close call enumeration at both callers"
                );
            }
        }
        let mut third = source.clone();
        let mut item = source.trait_impls[members[0]].clone();
        let mut parsed = ast::<syn::ItemImpl>(&item.1);
        parsed.self_ty = if domain.trait_path == profile.trait_path {
            Box::new(syn::parse_quote!(
                crate::runtime::accounting_mode::counter::Unarmed
            ))
        } else {
            Box::new(syn::parse_quote!(
                crate::runtime::suspension::core::profile_mode::Ordinary
            ))
        };
        item.1 = tokens(&parsed);
        third.trait_impls.push(item);
        assert!(
            domain.check(&third).is_err(),
            "a third nominal impl closes the domain"
        );
        if domain.trait_path == profile.trait_path {
            assert!(
                callers.into_iter().all(|caller| {
                    methods::claims(&third, caller)
                        .into_values()
                        .filter_map(Result::ok)
                        .all(|s| s.function.id != target)
                }),
                "third impl must close call enumeration at both callers"
            );
        }
        let mut conditional = source.clone();
        conditional.trait_impls[members[0]].3 = true;
        assert!(domain.check(&conditional).is_err());
        let mut generic = source.clone();
        let mut parsed = ast::<syn::ItemImpl>(&generic.trait_impls[members[0]].1);
        parsed.generics = syn::parse_quote!(<T>);
        generic.trait_impls[members[0]].1 = tokens(&parsed);
        assert!(domain.check(&generic).is_err());
    }
}

#[test]
fn finite_modes_and_opaque_brands() {
    let source = source::test_module(
        source::production().to_owned(),
        "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs",
        "rum_runtime::runtime::accounting_mode::tests",
    );
    for name in ["start", "drive", "resume"] {
        let f = resolve::function(
            &source,
            &format!("rum_runtime::runtime::accounting_mode::tests::{name}"),
        )
        .unwrap();
        let mut instances = BTreeSet::new();
        for id in types::MODES {
            let mode = ExactTy::Nominal(id.into(), Vec::new());
            let instance = types::with_mode(&source, f, Some(&mode)).unwrap();
            let params = types::exact_parameters(&source, &instance).unwrap();
            let target = format!("{}::reserve", protocol::CONTEXT);
            let method = types::with_mode(
                &source,
                protocol::signature(&source, &target).unwrap(),
                Some(&mode),
            )
            .unwrap();
            assert!(
                protocol::method(&source, &params[0].1, "reserve").is_some(),
                "receiver={:?}; params={:?}; output={:?}",
                params[0].1,
                types::exact_parameters(&source, &method),
                types::exact_output(&source, &method)
            );
            assert!(matches!(&params[0].1, ExactTy::Nominal(_, args) if args == &[mode]));
            instances.insert(types::exact_output(&source, &instance).unwrap());
        }
        assert_eq!(
            instances.len(),
            2,
            "both sealed substitutions must be checked"
        );
        let mut open = f.clone();
        open.sig = rewrite_generics(&open.sig, "<T>");
        assert!(types::with_mode(&source, &open, None).is_err());
        assert!(types::with_mode(&source, f, Some(&ExactTy::Scalar("u8".into()))).is_err());
    }
    let module = "rum_runtime::runtime::accounting_mode::tests";
    let mut changed = source.clone();
    changed.trait_impls.retain(|(m, item, _, _)| {
        ast::<syn::ItemImpl>(item)
            .trait_
            .as_ref()
            .is_none_or(|(_, p, _)| {
                resolve::path(&source, m, p).ok().as_deref()
                    != Some("rum_runtime::runtime::accounting_mode::lease::Mode")
            })
    });
    assert!(
        types::domain(&changed).is_err(),
        "missing finite mode implementation must close enumeration"
    );
    assert!(
        protocol::method(
            &source,
            &protocol::borrow(
                protocol::borrow(protocol::nominal(protocol::ARGS), false),
                false
            ),
            "clone"
        )
        .is_none()
    );
    let a: syn::Type = syn::parse_quote!(&'id SharedArgs);
    let b: syn::Type = syn::parse_quote!(&SharedArgs);
    assert_eq!(
        types::exact(&source, module, &a),
        types::exact(&source, module, &b)
    );
}
