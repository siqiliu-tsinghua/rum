use super::super::source;
use super::*;

// Explicit maintenance command, never automatic admission. Review source facts
// first, then run every recorded falsifier against the regenerated inventory.
#[test]
#[ignore = "writes a review copy; does not discharge contracts or install marks"]
fn regenerate_local_contract_fingerprints() {
    let input = include_str!("../inventory/local-contracts.tsv");
    let records = contracts::read(input).unwrap();
    let mut output = String::new();
    for line in input.lines() {
        if line.is_empty() || line.starts_with('#') {
            output.push_str(line);
        } else {
            let (fields, _) = line.rsplit_once('\t').unwrap();
            let id = fields.split('\t').next().unwrap();
            let record = records.iter().find(|record| record.id == id).unwrap();
            output.push_str(fields);
            output.push('\t');
            output.push_str(&record.current);
        }
        output.push('\n');
    }
    let directory = source::workspace().join("target/brief-544");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("local-contracts.tsv");
    std::fs::write(&path, output).unwrap();
    println!(
        "Regenerated {} fingerprints at {}",
        records.len(),
        path.display()
    );
}

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
    let index = Interfaces::shared_production();
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
