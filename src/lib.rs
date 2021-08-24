#![cfg_attr(not(feature = "std"), no_std)]

#[cfg_attr(not(feature = "std"), macro_use)]
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

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
type FxBuildHasher = BuildHasherDefault<FxHasher>;

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

/**
Need for complex Enums, which includes other data:
```
enum Complex {
    A(String),
    B(u32),
    C
}
enum Primitive {
    A,
    B,
    C,
}
```
PrimitiveEnum should be equivalent for Complex, but without variants inner data
*/
pub trait PrimitiveFromEnum {
    type PrimitiveEnum;

    fn get_primitive_enum(&self) -> Self::PrimitiveEnum;
}
