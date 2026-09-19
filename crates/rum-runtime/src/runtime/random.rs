use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

use std::error::Error;
use std::fmt;
use std::sync::Arc;

const SPLITMIX64_INCREMENT: u64 = 0x9e37_79b9_7f4a_7c15;
const NEGATIVE_SEED_TAG: u64 = 0xd1b5_4a32_d192_ed03;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct PrngState {
    state: [u64; 4],
}

impl PrngState {
    pub(in crate::runtime) fn from_seed_u64(seed: u64) -> Self {
        let mut splitmix = SplitMix64::new(seed);
        Self {
            state: [
                splitmix.next_u64(),
                splitmix.next_u64(),
                splitmix.next_u64(),
                splitmix.next_u64(),
            ],
        }
    }

    pub(in crate::runtime) fn from_seed_integer(seed: &BigInt) -> Self {
        let zero = BigInt::from(0_u8);
        if seed == &zero {
            return Self::from_seed_u64(0);
        }

        let negative = seed < &zero;
        let magnitude = if negative {
            -seed.clone()
        } else {
            seed.clone()
        };
        let (_, bytes) = magnitude.to_bytes_le();
        Self::from_seed_u64(fold_seed_bytes(&bytes, negative))
    }

    pub(in crate::runtime) fn from_seed_bytes(bytes: &[u8]) -> Self {
        Self::from_seed_u64(fold_seed_bytes(bytes, false))
    }

    pub(in crate::runtime) fn next_u64(&mut self) -> u64 {
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }

    pub(in crate::runtime) fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let word = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
    }
}

impl Default for PrngState {
    fn default() -> Self {
        Self::from_seed_u64(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX64_INCREMENT);
        splitmix64_mix(self.state)
    }
}

fn splitmix64_mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn fold_seed_bytes(bytes: &[u8], negative: bool) -> u64 {
    let mut folded = 0_u64;

    for chunk in bytes.chunks(8) {
        let mut word = [0_u8; 8];
        word[..chunk.len()].copy_from_slice(chunk);
        folded =
            splitmix64_mix(folded ^ u64::from_le_bytes(word).wrapping_add(SPLITMIX64_INCREMENT));
    }

    if negative {
        folded = splitmix64_mix((folded ^ NEGATIVE_SEED_TAG).wrapping_add(SPLITMIX64_INCREMENT));
    }

    folded
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntropyError {
    message: String,
}

impl EntropyError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for EntropyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for EntropyError {}

pub trait EntropySource: super::host_payload::BoundedDestruction + Send + Sync {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError>;
}

#[derive(Clone)]
pub struct EntropyProvider {
    inner: Arc<dyn EntropySource>,
}

impl EntropyProvider {
    pub fn new(provider: impl EntropySource + 'static) -> Self {
        Self {
            inner: Arc::new(provider),
        }
    }

    pub fn from_arc(provider: Arc<dyn EntropySource>) -> Self {
        Self { inner: provider }
    }

    pub fn as_entropy_source(&self) -> &dyn EntropySource {
        self.inner.as_ref()
    }
}

impl fmt::Debug for EntropyProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EntropyProvider")
    }
}

impl PartialEq for EntropyProvider {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for EntropyProvider {}
