//! Only the parent authenticator can construct a source capability.
use std::{collections::BTreeMap, marker::PhantomData, path::PathBuf};

mod sealed {
    pub trait Sealed {}
}

pub(crate) trait Arm: sealed::Sealed {
    const NAME: &'static str;
    const ENABLED: bool;
}

pub(crate) enum Disabled {}
pub(crate) enum Armed {}
impl sealed::Sealed for Disabled {}
impl sealed::Sealed for Armed {}
impl Arm for Disabled {
    const NAME: &'static str = "disabled";
    const ENABLED: bool = false;
}
impl Arm for Armed {
    const NAME: &'static str = "armed";
    const ENABLED: bool = true;
}

pub(crate) struct AuthenticatedSource<A: Arm> {
    pub(super) root: PathBuf,
    pub(super) originals: BTreeMap<String, Vec<u8>>,
    pub(super) postimages: BTreeMap<String, String>,
    pub(super) arm: PhantomData<A>,
}

/// Only source authentication can admit this row's armed observation mode.
pub(crate) struct MutationObserver<'a> {
    pub(super) source: &'a AuthenticatedSource<Armed>,
    pub(super) row: &'static str,
}
