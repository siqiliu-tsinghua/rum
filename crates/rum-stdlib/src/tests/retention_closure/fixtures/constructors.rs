use super::accepted::substrate::{self, Retained};
use std::option::Option as Maybe;
use std::option::Option::{None as Empty, Some as Present};
use std::result::Result::{Err as Failure, Ok as Success};

pub struct Pair {
    pub first: Retained,
    pub second: Retained,
}

pub struct UnleasedSibling {
    pub first: Retained,
    pub second: Vec<u8>,
}

pub fn root() -> Option<Retained> {
    Some(substrate::adopt(substrate::callback(1usize)))
}

pub fn aliases(value: Maybe<Retained>) -> Result<Maybe<Retained>, usize> {
    match value {
        Present(payload) => Success(Present(payload)),
        Empty => Failure(1usize),
    }
}

pub fn let_else(value: Option<Retained>) -> Option<Retained> {
    let Some(payload) = value else {
        return None;
    };
    Some(payload)
}

pub fn explicit_return(value: Result<Retained, usize>) -> Result<Retained, usize> {
    match value {
        Ok(payload) => return Ok(payload),
        Err(error) => Err(error),
    }
}

pub fn partial(value: Pair) -> (Retained, Retained) {
    let first = value.first;
    (first, value.second)
}

pub fn partial_bad(value: UnleasedSibling) -> Retained {
    let first = value.first;
    first
}

pub fn callback_bad() -> Option<substrate::Raw> {
    Some(substrate::callback(1usize))
}

pub fn raw_bad(value: Vec<u8>) -> Option<Vec<u8>> {
    Some(value)
}

pub fn nested_bad() -> Result<Option<substrate::Raw>, usize> {
    Ok(Some(substrate::callback(1usize)))
}

pub fn borrowed(value: &Pair) -> &Retained {
    &value.first
}

pub fn annotated(value: Option<usize>) -> Option<usize> {
    let exact: Option<usize> = value;
    exact
}

pub fn qualified(value: usize) -> Result<Option<usize>, u8> {
    ::core::result::Result::<Option<usize>, u8>::Ok(std::option::Option::Some(value))
}

pub fn empty() -> Option<usize> {
    Option::<usize>::None
}

pub fn guarded(value: Option<usize>) -> Option<usize> {
    match value {
        Some(payload) if payload == 1usize => Some(payload),
        _ => None,
    }
}

pub fn shadowed(value: usize) -> Option<usize> {
    let value = value;
    Some(value)
}

#[allow(non_snake_case)]
pub mod local {
    pub fn Some(value: usize) -> Option<usize> {
        std::option::Option::Some(value.clone())
    }
    pub fn Ok(value: usize) -> Result<usize, u8> {
        std::result::Result::Ok(value.clone())
    }
    pub fn some_root() -> Option<usize> {
        Some(1usize)
    }
    pub fn ok_root() -> Result<usize, u8> {
        Ok(1usize)
    }
}

pub fn known_type_rejected_effect(value: &usize) -> Result<usize, u8> {
    Ok(value.clone())
}

pub fn keep<T>(value: T) -> T {
    value
}

pub fn instances() -> (u8, usize) {
    (keep::<u8>(1u8), keep::<usize>(1usize))
}

pub struct Wrapper<T> {
    pub item: T,
    pub flag: bool,
}

pub enum Payload {
    Ready { item: Retained },
    Empty,
}

pub fn named_generic(value: Wrapper<Retained>) -> Retained {
    value.item
}

pub fn named_variant(value: Payload) -> Option<Retained> {
    match value {
        Payload::Ready { item } => Some(item),
        Payload::Empty => None,
    }
}

pub fn branch_mismatch(value: Pair, flag: bool) -> Retained {
    if flag {
        let _first = value.first;
    } else {
    }
    value.second
}

pub fn borrowed_pattern(value: &Option<Retained>) -> Option<&Retained> {
    match value {
        Some(item) => Some(item),
        None => None,
    }
}

pub fn nested_references(value: &&Option<usize>) -> bool {
    match value {
        Some(_item) => true,
        None => false,
    }
}

pub struct Cleanup {
    pub value: usize,
}

impl Drop for Cleanup {
    fn drop(&mut self) {}
}

pub fn unchecked_cleanup(value: Option<Cleanup>) -> Option<usize> {
    match value {
        Some(payload) => Some(payload.value),
        None => None,
    }
}

type Swap<A, B> = Result<B, A>;

pub fn reordered_alias() -> Swap<u8, usize> {
    Swap::<u8, usize>::Ok(1usize)
}

pub fn explicit_prelude() -> Option<usize> {
    std::prelude::rust_2024::Some(1usize)
}

pub mod globbed {
    use std::option::Option::*;
    pub fn root(value: Option<usize>) -> Option<usize> {
        match value {
            Some(item) => Some(item),
            None => None,
        }
    }
}
