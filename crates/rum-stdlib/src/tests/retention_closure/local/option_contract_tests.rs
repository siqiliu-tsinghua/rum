use super::super::super::source;
use super::*;

#[test]
fn local_option_contract_controls() {
    let source = source::production();
    let index = Interfaces::shared_production();
    let records = contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap();
    let marks = Marks::contracts(records.clone()).unwrap();
    let body = resolve::function(&source, &c01()).unwrap();
    let green = check(&index, body, &marks);
    assert!(green.rejected.is_empty(), "{}", green.text());
    for record in records
        .iter()
        .filter(|r| extra_targets().contains_key(&r.id))
    {
        for pending in [false, true] {
            let mut changed = records.clone();
            let row = changed.iter_mut().find(|r| r.id == record.id).unwrap();
            if pending {
                row.status = "Pending".into();
            } else {
                row.fingerprint.push('0');
            }
            assert!(
                !check(&index, body, &Marks::contracts(changed).unwrap())
                    .rejected
                    .is_empty()
            );
        }
    }
    for target in [SNAPSHOT, DRIVE] {
        let mut changed = source.clone();
        changed.functions.get_mut(target).unwrap()[0].body =
            block("{ panic!(\"omitted boundary\") }");
        assert!(validate(&changed).values().all(Result::is_err));
    }
    let mut schema = source.clone();
    schema.types.get_mut(OWNER).unwrap()[0].fields[0].name = "raw".into();
    assert!(validate(&schema).values().all(Result::is_err));
    let mut descriptor = source.clone();
    let id = option_domain::compiled_members()[0].spelling();
    descriptor.types.get_mut(&id).unwrap()[0]
        .variants
        .pop_first();
    assert!(validate(&descriptor).values().all(Result::is_err));
    let buffer = "rum_runtime::runtime::accounting_mode::options::buffer::OptionBuffer";
    for mutation in 0..3 {
        let mut changed = source.clone();
        let variants = &mut changed.types.get_mut(buffer).unwrap()[0].variants;
        match mutation {
            0 => {
                variants.remove("Flat");
            }
            1 => variants.get_mut("Flat").unwrap()[0].ty = type_text("usize"),
            _ => variants.get_mut("Flat").unwrap()[0].conditional = true,
        }
        assert!(validate(&changed).values().all(Result::is_err));
    }
    let mut changed = source.clone();
    let (_, mode, _, _) = changed
        .trait_impls
        .iter_mut()
        .find(|(module, item, _, _)| {
            let item = ast::<syn::ItemImpl>(item);
            module == "rum_runtime::runtime::accounting_mode::lease"
                && tokens(&item.self_ty) == "Unarmed"
                && item.items.iter().any(
                    |item| matches!(item, syn::ImplItem::Fn(f) if f.sig.ident == "option_handoff"),
                )
        })
        .unwrap();
    let mut parsed = ast::<syn::ItemImpl>(mode);
    for item in &mut parsed.items {
        if let syn::ImplItem::Fn(f) = item {
            if f.sig.ident == "option_handoff" {
                f.block = syn::parse_quote!({ true });
            }
        }
    }
    *mode = tokens(&parsed);
    assert!(validate(&changed).values().all(Result::is_err));
    // Persistent replacements for the former raw Vec at a shifting source line.
    let mut raw = body.clone();
    raw.body = r#"{
        let raw = std::vec::Vec::<rum_core::Expr>::new();
        let state = crate::runtime::accounting_mode::options::snapshot::<M>(self, symbol, explicit)?;
        crate::runtime::accounting_mode::options::drive::<M, R>(self, state, symbol, explicit, 0, reentry, head, args)
    }"#
    .into();
    let raw_report = check(&index, &raw, &marks);
    assert!(
        raw_report.rejected.iter().any(|d| d.kind == Kind::RET001),
        "{}",
        raw_report.text()
    );
    let domain = index.option_domain.as_ref().unwrap();
    for mode in types::MODES {
        for r in domain {
            let instance = option_domain::specialize(
                &index.signatures,
                body,
                &[protocol::nominal(mode), r.clone()],
                domain,
            )
            .unwrap();
            assert!(
                check_instance(&index, &instance, &marks)
                    .rejected
                    .is_empty()
            );
        }
    }
    assert!(
        option_domain::specialize(
            &index.signatures,
            body,
            &[
                protocol::nominal(types::MODES[0]),
                protocol::nominal("outside::HostType")
            ],
            domain
        )
        .is_err()
    );
    assert!(
        !check_instance(&index, body, &marks).rejected.is_empty(),
        "all-R cannot acquire a certificate"
    );
    // No implicit unsizing or extra reference layer is admitted at the adapter.
    let mut wrong = body.clone();
    wrong.body = block(
        "{
        crate::runtime::accounting_mode::options::snapshot::<M>(&self, symbol, explicit)
    }",
    );
    assert!(!check(&index, &wrong, &marks).rejected.is_empty());
    println!(
        "OPTION544_CONTRACT: concrete M x R green; stale/Pending/body/schema/raw-container/outside-R/all-R/wrong-reborrow red; promotion=0"
    );
}
