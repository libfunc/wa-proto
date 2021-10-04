#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod protocol;

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
use core::hash::BuildHasherDefault;
#[cfg(all(not(feature = "std"), feature = "hashmap"))]
use hashbrown::HashMap;
#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
use rustc_hash::FxHasher;
#[cfg(feature = "std")]
use std::collections::HashMap;

pub use protocol::*;
pub use wa_proto_macro::*;

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
type FxBuildHasher = BuildHasherDefault<FxHasher>;

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
