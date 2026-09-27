use super::super::source;
use super::*;

#[test]
fn local_concrete_contract_controls() {
    use rum_core::{Expr, SharedArgs};
    use std::ops::{Deref, Index};
    // These are observations of the stated finite cases, not an allocation
    // bound for arbitrary inputs or a certificate for deep Expr destruction.
    let mut raw = Vec::with_capacity(32);
    raw.push(Expr::small_integer(1));
    raw.push(Expr::string("prefix538".repeat(3)));
    let args = SharedArgs::from(raw);
    let view = <SharedArgs as Deref>::deref(&args);
    assert_eq!(view.capacity(), 32);
    assert_eq!(view.len(), 2);
    let first = <Vec<Expr> as Index<usize>>::index(view, 0);
    assert!(std::ptr::eq(first, &args[0]));
    assert!(<Expr as PartialEq>::eq(first, &Expr::small_integer(1)));
    assert!(!<Expr as PartialEq>::eq(first, &Expr::small_integer(2)));
    assert_eq!(<Expr as Clone>::clone(first), *first);
    let shared = <SharedArgs as Clone>::clone(&args);
    assert!(SharedArgs::ptr_eq(&args, &shared));
    let records = contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap();
    let source = source::production();
    let index = Interfaces::new(&source);
    for record in records.iter().filter(|r| r.status == "Discharged") {
        let good = Marks::contracts(records.clone()).unwrap();
        assert!(good.lookup(&index, &record.target).is_ok());
        let mut changed = record.clone();
        changed.fingerprint.push('0');
        let bad = Marks::contracts(vec![changed]).unwrap();
        let red = bad.lookup(&index, &record.target).unwrap_err();
        assert!(red.contains("stale contract fingerprint"));
        let dir = source::workspace().join("target/brief-540/falsifiers");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{}.red-green.txt", record.id)),
            format!("RED {red}\nGREEN exact current mark\n"),
        )
        .unwrap();
    }
}
