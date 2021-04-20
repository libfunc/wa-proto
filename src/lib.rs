#![cfg_attr(not(feature = "std"), no_std)]

#[cfg_attr(not(feature = "std"), macro_use)]
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
use rustc_hash::{FxHasher};
#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
use core::hash::BuildHasherDefault;
#[cfg(all(not(feature = "std"), feature = "hashmap"))]
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::collections::HashMap;

pub mod protocol;

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
pub type FxBuildHasher = BuildHasherDefault<FxHasher>;

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

pub trait PrimitiveFromEnum {
    type PrimitiveEnum;

    fn get_primitive_enum(&self) -> Self::PrimitiveEnum;
}
