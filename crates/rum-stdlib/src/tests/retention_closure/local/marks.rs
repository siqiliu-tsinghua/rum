//! Only a successful local run can mint a covered mark. Inventory text cannot.
//! Validation is a flat pass over marks and their direct dependency identities.

use super::super::model::*;
use super::{Report, contracts::Contract, interface::Interfaces};
use std::collections::BTreeMap;

#[derive(Clone)]
struct Covered {
    fingerprint: String,
    dependencies: BTreeMap<Id, String>,
    instance: Option<super::certificates::Instance>,
}

#[derive(Clone, Default)]
pub(super) struct Marks {
    covered: BTreeMap<Id, Covered>,
    contracts: BTreeMap<Id, Contract>,
}

impl Marks {
    pub fn contracts(contracts: Vec<Contract>) -> Result<Self, String> {
        let mut result = Self::default();
        for record in contracts {
            if result
                .contracts
                .insert(record.target.clone(), record)
                .is_some()
            {
                return Err("duplicate contract identity".into());
            }
        }
        Ok(result)
    }

    fn fingerprint(&self, id: &str) -> Option<&str> {
        self.covered
            .get(id)
            .map(|m| m.fingerprint.as_str())
            .or_else(|| self.contracts.get(id).map(|c| c.fingerprint.as_str()))
    }

    pub fn lookup(&self, index: &Interfaces, id: &str) -> Result<String, String> {
        self.view(index)
            .remove(id)
            .unwrap_or_else(|| Self::missing(id))
    }

    pub fn missing(id: &str) -> Result<String, String> {
        Err(format!(
            "callee has no covered mark or installed contract: {id}"
        )) // FALSIFIER:missing-mark
    }

    fn validate_covered(&self, index: &Interfaces) -> Result<(), String> {
        // No body access here. Stamps were computed at indexing, before lint.
        // Validate the complete mark set, without following any call edges.
        for (name, mark) in &self.covered {
            if let Some(instance) = &mark.instance {
                instance.current(index)?;
                if instance.fingerprint() != mark.fingerprint {
                    return Err(format!("stale production instance {name}"));
                }
            } else if index.bodies.get(name) != Some(&mark.fingerprint) {
                return Err(format!("stale covered mark {name}; re-lint its own body"));
            } // FALSIFIER:covered-fingerprint
            for (dependency, stamp) in &mark.dependencies {
                if let Some(contract) = self.contracts.get(dependency) {
                    // A matching stamp is insufficient after a dependency's
                    // status or indexed declaration evidence was withdrawn.
                    self.contract_view(index, contract)?;
                }
                if self.fingerprint(dependency) != Some(stamp.as_str()) {
                    return Err(format!(
                        "changed direct mark dependency {name} -> {dependency}"
                    ));
                }
            }
        }
        // A discharged contract's expiry invalidates the prepared mark set.
        // This flat check avoids recursive dependency or body traversal.
        for record in self.contracts.values().filter(|c| c.status != "Pending") {
            Self::contract(record)?;
        }
        Ok(())
    }

    fn contract(record: &Contract) -> Result<String, String> {
        if record.status == "Pending" {
            return Err(format!("Pending contract {} is not a mark", record.id));
        } // FALSIFIER:pending
        if record.fingerprint != record.current {
            return Err(format!("stale contract fingerprint {}", record.id));
        } // FALSIFIER:contract-fingerprint
        if let Some(error) = &record.invalid {
            return Err(error.clone());
        }
        // The mark does not discharge caller preconditions or resolution.
        return Ok(record.fingerprint.clone());
    }

    fn contract_view(&self, index: &Interfaces, c: &Contract) -> Result<String, String> {
        let id = &c.target;
        Self::contract(c).and_then(|stamp| {
            if let Some(check) = index.read_contracts.get(id) {
                check.clone()?;
            }
            // The fixed rule protocol's direct records are reconciled
            // without recursively asking another mark or body.
            for dependency in super::rules::dependencies(id)
                .into_iter()
                .chain(super::option_boundaries::dependencies(id))
            {
                let record = self
                    .contracts
                    .get(&dependency)
                    .ok_or_else(|| format!("missing rule boundary dependency {dependency}"))?;
                Self::contract(record)?;
                if let Some(check) = index.read_contracts.get(&dependency) {
                    check.clone()?;
                }
            }
            Ok(stamp)
        })
    }

    pub fn view(&self, index: &Interfaces) -> BTreeMap<Id, Result<String, String>> {
        let validation = self.validate_covered(index);
        let mut view = self
            .contracts
            .iter()
            .map(|(id, c)| (id.clone(), self.contract_view(index, c)))
            .collect::<BTreeMap<_, _>>();
        for (id, m) in &self.covered {
            view.insert(
                id.clone(),
                validation.clone().map(|()| m.fingerprint.clone()),
            );
        }
        view
    }

    pub fn certify(&mut self, index: &Interfaces, f: &Function) -> Result<Report, Report> {
        let mut report = super::check(index, f, self);
        if !index.matches(f) {
            report.rejected.insert(Diagnostic {
                kind: Kind::RET002,
                site: f.site.clone(),
                target: Some(f.id.clone()),
                witness: vec![f.id.clone()],
                reason: "linted body differs from indexed fingerprint".into(),
            });
        }
        if !report.rejected.is_empty() {
            return Err(report);
        }
        let Some(fingerprint) = index.bodies.get(&f.id) else {
            return Err(report);
        };
        if super::certificates::production(f) {
            let instances = match super::certificates::instances(index, f) {
                Ok(instances) => instances,
                Err(reason) => {
                    report.rejected.insert(Diagnostic {
                        kind: Kind::RET002,
                        site: f.site.clone(),
                        target: Some(f.id.clone()),
                        witness: vec![f.id.clone()],
                        reason,
                    });
                    return Err(report);
                }
            };
            let mut earned = Vec::new();
            for (instance, body) in instances {
                let own = super::check_instance(index, &body, &self.with_covered(&index.earned));
                if !own.rejected.is_empty() {
                    return Err(own);
                }
                earned.push((
                    instance.key.clone(),
                    Covered {
                        fingerprint: instance.fingerprint(),
                        dependencies: own.marks,
                        instance: Some(instance),
                    },
                ));
            }
            // Atomic publication: an omitted or rejected instance earns nothing.
            self.covered.extend(earned);
            return Ok(report);
        }
        if !f.id.starts_with("fixture::") && !super::interface::probe_body(f) {
            return Err(report);
        }
        self.covered.insert(
            f.id.clone(),
            Covered {
                fingerprint: fingerprint.clone(),
                dependencies: report.marks.clone(),
                instance: None,
            },
        );
        Ok(report)
    }

    pub fn production_records(
        &self,
        index: &Interfaces,
    ) -> Result<Vec<(&super::certificates::Instance, &BTreeMap<Id, String>)>, String> {
        self.validate_covered(index)?;
        Ok(self
            .covered
            .values()
            .filter_map(|m| m.instance.as_ref().map(|i| (i, &m.dependencies)))
            .collect())
    }

    pub fn count(&self) -> usize {
        self.covered.len()
    }

    pub fn inert(&self, id: &str) -> bool {
        self.contracts
            .get(id)
            .is_some_and(|c| c.inert && Self::contract(c).is_ok())
    }

    pub fn inert_targets(&self) -> std::collections::BTreeSet<Id> {
        self.contracts
            .keys()
            .filter(|id| self.inert(id))
            .cloned()
            .collect()
    }

    pub fn with_covered(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.covered.extend(other.covered.clone());
        result
    }

    pub fn inventory(&self) -> String {
        self.covered
            .iter()
            .map(|(id, m)| format!("{id}\t{}\t{:?}\n", m.fingerprint, m.dependencies))
            .collect()
    }
}
