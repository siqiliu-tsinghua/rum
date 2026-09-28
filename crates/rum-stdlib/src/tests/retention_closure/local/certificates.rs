//! Finite production instance identities. No all-R mark and no call traversal.
use super::super::{regions, source};
use super::*;

pub(super) const DESCRIPTOR: &str = "rum_stdlib::option_consumers::OptionConsumer";
pub(super) const FILE: &str = "crates/rum-runtime/src/runtime/options/core.rs";
pub(super) const EXCLUSIONS: &str = "local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged";
pub(super) const PRECONDITIONS: &str = "exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required";
pub(super) const STRENGTH: &str = "local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type";

pub(super) fn roots() -> [(String, String); 2] {
    [
        ("C01".into(), option_boundaries::c01()),
        (
            "C02".into(),
            format!("{}::evaluate_runtime_option_expr", regions::SESSION),
        ),
    ]
}

pub(super) fn production(f: &Function) -> bool {
    roots().iter().any(|(_, id)| id == &f.id)
}

pub(super) fn build() -> String {
    static BUILD: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BUILD.get_or_init(|| {
        let output = std::process::Command::new("python3")
            .arg(source::workspace().join("scripts/run-bounded.py"))
            .args(["--timeout", "30", "--", "rustc", "-vV"])
            .output().expect("read the exact compiler identity");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let compiler = String::from_utf8(output.stdout).unwrap().trim().replace('\n', ";");
        let manifests = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "crates/rum-runtime/Cargo.toml", "crates/rum-stdlib/Cargo.toml"]
            .map(|file| std::fs::read(source::workspace().join(file)).unwrap());
        format!("target={}-{};pointer={};debug={};hash={};compiler={compiler};build-inputs={};rustflags={:?}",
            std::env::consts::ARCH, std::env::consts::OS, usize::BITS,
            cfg!(debug_assertions), cfg!(feature = "hash"), interface::hash(manifests),
            option_env!("RUSTFLAGS"))
    }).clone()
}

pub(super) fn domain(source: &Source) -> Result<String, String> {
    let members = option_domain::members(source)?;
    if members != [protocol::nominal(DESCRIPTOR)] {
        return Err("production certificates require the one repository R declaration".into());
    }
    types::domain(source)?;
    let mut session = rum_runtime::Session::new();
    crate::install_core(&mut session);
    option_domain::reconcile(source, &session, &option_domain::keys(&members))?;
    // Flat source dependencies for the declaration, selection, installer and
    // runtime enforcement. They are stamps, never bodies traversed by lint.
    let mut parts = vec![format!("{:?}", option_domain::keys(&members))];
    for file in [
        option_domain::DECLARATION,
        "crates/rum-stdlib/src/lib.rs",
        "crates/rum-stdlib/src/option_consumers.rs",
        "crates/rum-runtime/src/runtime/options/reentry.rs",
        "crates/rum-runtime/src/runtime/accounting_mode/lease.rs",
        "crates/rum-runtime/src/runtime/accounting_mode/counter.rs",
    ] {
        parts.push(file.into());
        parts.push(
            source
                .files
                .get(file)
                .ok_or_else(|| format!("absent domain source {file}"))?
                .clone(),
        );
    }
    Ok(interface::hash(parts))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Instance {
    pub body: Id,
    pub key: Id,
    pub mode: String,
    pub reentry: String,
    pub build: String,
    pub body_fingerprint: String,
    pub domain_fingerprint: String,
}

impl Instance {
    pub fn fingerprint(&self) -> String {
        interface::hash([format!("{self:?}")])
    }

    pub fn current(&self, index: &Interfaces) -> Result<(), String> {
        if index.bodies.get(&self.body) != Some(&self.body_fingerprint) {
            return Err(format!("expired production body fingerprint {}", self.key));
        }
        if index.certificate_domain.as_ref() != Ok(&self.domain_fingerprint) {
            return Err(format!(
                "expired production domain fingerprint {}",
                self.key
            ));
        }
        if index.build != self.build {
            return Err(format!("expired production build {}", self.key));
        }
        Ok(())
    }
}

pub(super) fn instances(
    index: &Interfaces,
    f: &Function,
) -> Result<Vec<(Instance, Function)>, String> {
    if !production(f) || f.site.file != FILE || !index.matches(f) || !f.substitutions.is_empty() {
        return Err("production certificate requires its exact indexed unspecialized body".into());
    }
    let domain_fingerprint = index.certificate_domain.clone()?;
    let body_fingerprint = index
        .bodies
        .get(&f.id)
        .ok_or("absent production body fingerprint")?
        .clone();
    let members = index.option_domain.as_ref().map_err(Clone::clone)?;
    if members != &[protocol::nominal(DESCRIPTOR)] {
        return Err("missing or changed declared production instance".into());
    }
    let mut result = vec![];
    if f.id == option_boundaries::c01() {
        for mode in types::MODES {
            for r in members {
                let arguments = [protocol::nominal(mode), r.clone()];
                let specialized =
                    option_domain::specialize(&index.signatures, f, &arguments, members)?;
                result.push((
                    Instance {
                        body: f.id.clone(),
                        key: option_boundaries::key(&f.id, &arguments),
                        mode: mode.into(),
                        reentry: r.spelling(),
                        build: index.build.clone(),
                        body_fingerprint: body_fingerprint.clone(),
                        domain_fingerprint: domain_fingerprint.clone(),
                    },
                    specialized,
                ));
            }
        }
    } else {
        if !f.sig.generics.params.is_empty() || !f.impl_generics.params.is_empty() {
            return Err("C02 must remain nongeneric".into());
        }
        result.push((
            Instance {
                body: f.id.clone(),
                key: f.id.clone(),
                mode: format!(
                    "runtime dispatch via rule contracts: {}",
                    types::MODES.join(",")
                ),
                reentry: "none (nongeneric C02)".into(),
                build: index.build.clone(),
                body_fingerprint,
                domain_fingerprint,
            },
            f.clone(),
        ));
    }
    Ok(result)
}
