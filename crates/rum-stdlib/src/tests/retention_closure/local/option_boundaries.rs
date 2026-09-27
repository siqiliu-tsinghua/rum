//! The two private option adapters, specialized only for reconciled M x R.
//! A type fact is not a mark: pending or stale adapter records still reject.
use super::*;

#[path = "option_contract_tests.rs"]
mod tests;

const FILES: &[(&str, &str, &str)] = &[
    (
        "crates/rum-runtime/src/runtime/accounting_mode/options.rs",
        "rum_runtime::runtime::accounting_mode::options",
        include_str!("../../../../../rum-runtime/src/runtime/accounting_mode/options.rs"),
    ),
    (
        "crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs",
        "rum_runtime::runtime::accounting_mode::options::driver",
        include_str!("../../../../../rum-runtime/src/runtime/accounting_mode/options/driver.rs"),
    ),
    (
        "crates/rum-runtime/src/runtime/options/reentry.rs",
        "rum_runtime::runtime::options::reentry",
        include_str!("../../../../../rum-runtime/src/runtime/options/reentry.rs"),
    ),
    (
        "crates/rum-runtime/src/runtime/options/resume_frames.rs",
        "rum_runtime::runtime::options::resume_frames",
        include_str!("../../../../../rum-runtime/src/runtime/options/resume_frames.rs"),
    ),
    (
        "crates/rum-stdlib/src/option_consumers.rs",
        "rum_stdlib::option_consumers",
        include_str!("../../../option_consumers.rs"),
    ),
];

pub(super) fn validate(source: &Source) -> BTreeMap<Id, Result<(), String>> {
    let verdict = option_domain::members(source)
        .map(|_| ())
        .and_then(|()| rules::declarations(source, FILES));
    extra_targets()
        .into_values()
        .map(|id| (id, verdict.clone()))
        .collect()
}

pub(super) fn dependencies(target: &str) -> Vec<String> {
    if !extra_targets().values().any(|t| t == target) {
        return vec![];
    }
    ["B03", "B04", "B09", "B12"]
        .map(contracts::target)
        .into_iter()
        .chain(rules::extra_targets().into_values())
        .chain(extra_targets().into_values().filter(|t| t != target))
        .collect()
}

pub(super) const OWNER: &str = "rum_runtime::runtime::accounting_mode::options::OptionState";
pub(super) const SNAPSHOT: &str = "rum_runtime::runtime::accounting_mode::options::snapshot";
pub(super) const DRIVE: &str = "rum_runtime::runtime::accounting_mode::options::driver::drive";

pub(super) fn c01() -> String {
    format!(
        "{}::merged_option_exprs_for_symbol",
        super::super::regions::SESSION
    )
}

pub(super) fn key(target: &str, arguments: &[ExactTy]) -> String {
    format!(
        "{target}::<{}>",
        arguments
            .iter()
            .map(ExactTy::spelling)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(super) fn extra_targets() -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for (m, mode) in types::MODES.iter().enumerate() {
        let mode = protocol::nominal(mode);
        result.insert(format!("O01{m}"), key(SNAPSHOT, &[mode.clone()]));
        for (r, descriptor) in option_domain::compiled_members().into_iter().enumerate() {
            result.insert(
                format!("O02{m}{r}"),
                key(DRIVE, &[mode.clone(), descriptor]),
            );
        }
    }
    result
}

pub(super) fn check_instances(index: &Interfaces, f: &Function, marks: &Marks) -> Report {
    let mut reports = Report::default();
    let domain = index.option_domain.as_ref().map_err(Clone::clone);
    let domain = match domain {
        Ok(domain) => domain,
        Err(reason) => {
            reports.rejected.insert(Diagnostic {
                kind: Kind::RET002,
                site: f.site.clone(),
                target: Some(f.id.clone()),
                witness: vec![f.id.clone()],
                reason,
            });
            return reports;
        }
    };
    for mode in types::MODES {
        for descriptor in domain {
            match option_domain::specialize(
                &index.signatures,
                f,
                &[protocol::nominal(mode), descriptor.clone()],
                domain,
            ) {
                Ok(instance) => {
                    let report = check_instance(index, &instance, marks);
                    reports.rejected.extend(report.rejected);
                    reports.facts.extend(report.facts);
                    reports.marks.extend(report.marks);
                    reports.calls.extend(report.calls);
                    reports.bindings.extend(report.bindings);
                    reports.targets.extend(report.targets);
                    reports.reborrows.extend(report.reborrows);
                }
                Err(reason) => {
                    reports.rejected.insert(Diagnostic {
                        kind: Kind::RET002,
                        site: f.site.clone(),
                        target: Some(f.id.clone()),
                        witness: vec![f.id.clone()],
                        reason,
                    });
                }
            }
        }
    }
    reports
}

impl Lint<'_> {
    pub(super) fn option_boundary(
        &mut self,
        c: &syn::ExprCall,
        p: &syn::ExprPath,
        target: &str,
    ) -> Option<ExactTy> {
        let arguments = (|| -> Result<Vec<ExactTy>, String> {
            let mut result = vec![];
            for segment in &p.path.segments {
                match &segment.arguments {
                    syn::PathArguments::None => {}
                    syn::PathArguments::AngleBracketed(args) if result.is_empty() => {
                        for arg in &args.args {
                            let syn::GenericArgument::Type(ty) = arg else {
                                return Err("exact option type argument required".into());
                            };
                            result.push(types::in_function(self.source(), self.f, ty)?);
                        }
                    }
                    _ => return Err("one positional option argument list required".into()),
                }
            }
            Ok(result)
        })();
        let signature = arguments.and_then(|arguments| {
            let domain = self.index.option_domain.as_ref().map_err(Clone::clone)?;
            let f = resolve::function(self.source(), target)?;
            let instance = option_domain::specialize(self.source(), f, &arguments, domain)?;
            Ok((arguments, instance))
        });
        let (arguments, signature) = match signature {
            Ok(v) => v,
            Err(reason) => return self.no(c.span(), &reason),
        };
        let parameters = types::exact_parameters(self.source(), &signature).ok()?;
        if parameters.len() != c.args.len() {
            return self.no(c.span(), "option adapter arity differs");
        }
        self.call_mark(c.span(), &key(target, &arguments));
        self.report.facts.insert(
            Site::new(&self.f.site.file, p.span()),
            ExactTy::Function(key(target, &arguments)),
        );
        for (arg, (_, parameter)) in c.args.iter().zip(&parameters) {
            let value = self.expr(arg, Some(parameter));
            if value.as_ref() != Some(parameter) {
                self.reject(
                    arg.span(),
                    Kind::RET002,
                    None,
                    "option adapter argument differs from concrete parameter",
                );
            } else if matches!(parameter, ExactTy::Reference(inner, true) if **inner == protocol::nominal(super::super::regions::SESSION))
            {
                self.report
                    .reborrows
                    .insert(Site::new(&self.f.site.file, arg.span()));
            }
        }
        types::exact_output(self.source(), &signature).ok()
    }
}
