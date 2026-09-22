//! Arithmetic specification only: no Session storage and no maintained ledger.

use std::mem::size_of;

/// Exact includes usize::MAX. Invalid means overflow/underflow lost information;
/// its ceiling projection saturates, and releases cannot make it look smaller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum Bytes {
    Exact(usize),
    Invalid,
}

impl Bytes {
    pub fn ceiling_bytes(self) -> usize {
        match self {
            Self::Exact(bytes) => bytes,
            Self::Invalid => usize::MAX,
        }
    }

    pub fn exceeds(self, limit: usize) -> bool {
        match self {
            Self::Exact(bytes) => bytes > limit,
            Self::Invalid => true,
        }
    }

    pub fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Exact(left), Self::Exact(right)) => {
                left.checked_add(right).map_or(Self::Invalid, Self::Exact)
            }
            _ => Self::Invalid,
        }
    }

    pub fn multiply(self, count: usize) -> Self {
        match self {
            Self::Exact(bytes) => bytes.checked_mul(count).map_or(Self::Invalid, Self::Exact),
            Self::Invalid => Self::Invalid,
        }
    }

    /// Never use saturated subtraction for releases. MAX may be a lossy charge;
    /// subtracting from it could understate the remaining live occurrences.
    /// Only a fresh independent walk can recover an Invalid state.
    pub fn release(self, amount: Self) -> Self {
        match (self, amount) {
            (Self::Exact(total), Self::Exact(amount)) => {
                total.checked_sub(amount).map_or(Self::Invalid, Self::Exact)
            }
            _ => Self::Invalid,
        }
    }
}

pub(super) fn sum(values: impl IntoIterator<Item = Bytes>) -> Bytes {
    // `sum::<usize>()` panics in checked builds and wraps in unchecked builds.
    // Every intermediate retains invalidity, even if the final projection is MAX.
    values.into_iter().fold(Bytes::Exact(0), Bytes::add)
}

pub(super) fn slots<T>(capacity: usize) -> Bytes {
    Bytes::Exact(capacity).multiply(size_of::<T>())
}
