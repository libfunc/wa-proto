//! Зачем разделили на Incoming и Outcoming?
//! Конечно чаще всего структуры будут имплементировать оба трейта.
//! Но иногда возможно какую то структуру будут передавать в wasm,
//! но из wasm она не будет передаваться никогда. Тогда для этой
//! структуры излишне имплементировать Outcoming.

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
use crate::{FxBuildHasher, FxHashMap};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
#[cfg(feature = "chrono")]
use chrono::{Date, DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Utc};
#[cfg(feature = "std")]
use core::fmt;
#[cfg(not(feature = "std"))]
use core::{convert::TryInto, slice::IterMut};
#[cfg(all(not(feature = "std"), feature = "hashmap"))]
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::{
    cell::RefMut,
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    error::Error,
    hash::Hash,
    slice::Iter,
    string::String,
    vec::Vec,
};

// TODO: alignment memory of 32 bits?

#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg(feature = "std")]
pub struct ProtocolError(pub String);

#[derive(Default)]
#[cfg(not(feature = "std"))]
pub struct ProtocolError(pub u32);

#[cfg(feature = "std")]
pub const ARGS_NEXT_ERROR: &str = "args next error";

#[cfg(not(feature = "std"))]
pub const ARGS_NEXT_ERROR: u32 = 2;

pub const BYTES_INTO_ARR4_ERROR: u32 = 3;
pub const BYTES_INTO_ARR8_ERROR: u32 = 4;
pub const MAP_INSERT_ERROR: u32 = 5;
pub const STRING_FROM_BYTES_ERROR: u32 = 6;

#[cfg(feature = "std")]
pub const ENUM_FROM_U32_ERROR: &str = "enum from u32 error";

#[cfg(not(feature = "std"))]
pub const ENUM_FROM_U32_ERROR: u32 = 7;

pub const TIME_PARSE_ERROR: u32 = 8;

#[cfg(feature = "std")]
impl From<&str> for ProtocolError {
    fn from(s: &str) -> Self {
        ProtocolError(s.to_string())
    }
}

#[cfg(feature = "std")]
impl From<std::array::TryFromSliceError> for ProtocolError {
    fn from(_: std::array::TryFromSliceError) -> Self {
        ProtocolError("slice len error".to_string())
    }
}

#[cfg(feature = "std")]
impl From<std::string::FromUtf8Error> for ProtocolError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        ProtocolError("utf-8 string bytes error".to_string())
    }
}

#[cfg(feature = "std")]
impl From<time::error::ComponentRange> for ProtocolError {
    fn from(_: time::error::ComponentRange) -> Self {
        ProtocolError("time from u32 error".to_string())
    }
}

#[cfg(not(feature = "std"))]
impl From<u32> for ProtocolError {
    fn from(s: u32) -> Self {
        ProtocolError(s)
    }
}

#[cfg(feature = "std")]
impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "Protocol error, can't transform")
        } else {
            write!(f, "Protocol error: {}", &self.0)
        }
    }
}

#[cfg(feature = "std")]
impl Error for ProtocolError {}

/**
Incoming trait (Deserializable) - (Входящее сообщение) если структура реализует этот трейт,
то значит что эту структуру можно передать в wasm.
Внутри структуры находится информация:
- о длине строки или массива
- о типе варианта перечисления
- и другие данные, которые помогут определить полностью сообщение.
Важно одно ограничение - Структура должна быть построена на основе перечислений,
а не на основе дженериков (или хранить внутри себя информацию о соответствущем дженерике)?
Зачем это ограничение, если мы уже реализовали трейт для HashMap и Vec с дженериками?

Будет в основном использоваться для передачи сообщений между wasm модулем и Rust рантаймом.
Для сериализации и десериализации в БД рекомендуется использовать serde.
*/
// #[cfg(all(target = "wasm32-unknown-unknown"))]
pub trait Incoming {
    /**
    Указывает на то что необходимо инициализировать данные, а затем заполнить
    если true - получается 2 шага
    если false - 1 шаг
     */
    const IS_NEED_INIT_FILL: bool = false;

    /**
    Инициализируем кусок памяти в wasm для последующего заполнения.
    Вызывается в wasm.
    */
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError>
    where
        Self: Sized;

    /**
    Добавление в аргументы вспомогательных данных,
    таких как длина строки/массива и др.
    Вызывается на хосте.
    */
    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError>;

    /**
    Заполнение данными инициализированный участок памяти.
    вызывается на хосте.
    Иногда инициализация не нужна, если значение можно полностью передать в аргументах,
    уместить в значении u32.
    */
    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError>;
}

/**
Outcoming trait (Serializable) - (Исходящее сообщение) если структура реализует этот трейт,
то эту структуру можно передать из wasm на хост.

Будет в основном использоваться для передачи сообщений между wasm модулем и Rust рантаймом.
Для сериализации и десериализации в БД рекомендуется использовать serde.
*/
pub trait Outcoming {
    /**
    Указывает на то что необходимо прочитать данные из памяти песочницы
     */
    const IS_NEED_READ: bool = false;

    /**
    Заполнение массива чисел вспомогательными данными,
    такими как длина строки или массива и др.
    Вызывается в wasm.
    */
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError>;

    /**
    Чтение данных из памяти wasm.
    Вызывается на хосте.
    */
    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError>
    where
        Self: Sized;
}

impl Incoming for bool {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el != 0))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(if *self { 1 } else { 0 });
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for bool {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(if *self { 1 } else { 0 });
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(el != 0)
    }
}

impl Incoming for u8 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el as u8))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for u8 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(el as u8)
    }
}

impl Incoming for i32 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el as i32))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for i32 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        Ok(*args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as i32)
    }
}

impl Incoming for i64 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el1: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let el2: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR8_ERROR))?;
        let e = i64::from_le_bytes(*d);
        Ok((0, e))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4].try_into()?;
        let arr2: &[u8; 4] = bytes[4..8].try_into()?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for i64 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arr2: &[u8; 4] = bytes[4..8]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el1: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        let el2: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into()?;
        let e = i64::from_le_bytes(*d);
        Ok(e)
    }
}

impl Incoming for u32 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for u32 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        Ok(*args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?)
    }
}

impl Incoming for u64 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el1: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let el2: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR8_ERROR))?;
        let e = u64::from_le_bytes(*d);
        Ok((0, e))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4].try_into()?;
        let arr2: &[u8; 4] = bytes[4..8].try_into()?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for u64 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arr2: &[u8; 4] = bytes[4..8]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el1: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        let el2: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into()?;
        let e = u64::from_le_bytes(*d);
        Ok(e)
    }
}

// only for wasm32 and runner target_pointer_width = "32"
// #[cfg(all(not(feature = "std"), target_pointer_width = "32"))]
impl Incoming for usize {
    // NOTE: for wasm64 required implement other fn
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((0, el as usize))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

// only for wasm32 and runner target_pointer_width = "32"
// #[cfg(all(not(feature = "std"), target_pointer_width = "32"))]
impl Outcoming for usize {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(el as usize)
    }
}

// only for wasm32 and runner target_pointer_width = "32"
impl Incoming for isize {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((0, el as isize))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

// only for wasm32 and runner target_pointer_width = "32"
impl Outcoming for isize {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(el as isize)
    }
}

impl Incoming for f32 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let bytes: [u8; 4] = el.to_le_bytes();
        let f = f32::from_le_bytes(bytes);
        Ok((0, f))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes = self.to_le_bytes();
        let u = u32::from_le_bytes(bytes);
        args.push(u);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for f32 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 4] = self.to_le_bytes();
        let u = u32::from_le_bytes(bytes);
        args.push(u);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        let bytes: [u8; 4] = el.to_le_bytes();
        let f = f32::from_le_bytes(bytes);
        Ok(f)
    }
}

impl Incoming for f64 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el1: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let el2: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR8_ERROR))?;
        let e = f64::from_le_bytes(*d);
        Ok((0, e))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4].try_into()?;
        let arr2: &[u8; 4] = bytes[4..8].try_into()?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

impl Outcoming for f64 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arr2: &[u8; 4] = bytes[4..8]
            .try_into()
            .map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let el1: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        let el2: u32 = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into()?;
        let e = f64::from_le_bytes(*d);
        Ok(e)
    }
}

#[cfg(feature = "chrono")]
impl Incoming for Duration {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, e) = i64::init(args)?;
        let duration = Duration::milliseconds(e);
        Ok((0, duration))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let e = self.num_milliseconds();
        e.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "chrono")]
impl Outcoming for Duration {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let e = self.num_milliseconds();
        e.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let e = i64::read(heap, args)?;
        Ok(Duration::milliseconds(e))
    }
}

#[cfg(feature = "chrono")]
impl Incoming for DateTime<Utc> {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, secs) = i64::init(args)?;
        let dt = Self::from_utc(NaiveDateTime::from_timestamp(secs, 0), Utc);
        Ok((0, dt))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let secs = self.timestamp();
        secs.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "chrono")]
impl Outcoming for DateTime<Utc> {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let secs = self.timestamp();
        secs.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let secs = i64::read(heap, args)?;
        Ok(Self::from_utc(NaiveDateTime::from_timestamp(secs, 0), Utc))
    }
}

#[cfg(feature = "chrono")]
impl Incoming for Date<Utc> {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, days) = i32::init(args)?;
        let d = Self::from_utc(NaiveDate::from_num_days_from_ce(days), Utc);
        Ok((0, d))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let days = self.num_days_from_ce();
        days.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "chrono")]
impl Outcoming for Date<Utc> {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let days = self.num_days_from_ce();
        days.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let days = i32::read(heap, args)?;
        Ok(Self::from_utc(NaiveDate::from_num_days_from_ce(days), Utc))
    }
}

#[cfg(feature = "time")]
impl Incoming for time::Duration {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, e) = i64::init(args)?;
        let duration = time::Duration::seconds(e);
        Ok((0, duration))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let e = self.whole_seconds();
        e.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "time")]
impl Outcoming for time::Duration {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let e = self.whole_seconds();
        e.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let e = i64::read(heap, args)?;
        Ok(time::Duration::seconds(e))
    }
}

#[cfg(feature = "time")]
impl Incoming for time::OffsetDateTime {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, secs) = i64::init(args)?;
        let dt = Self::from_unix_timestamp(secs);
        match dt {
            Ok(dt) => Ok((0, dt)),
            Err(_) => Err(ProtocolError(TIME_PARSE_ERROR)),
        }
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let secs = self.unix_timestamp();
        secs.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "time")]
impl Outcoming for time::OffsetDateTime {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let secs = self.unix_timestamp();
        secs.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let secs = i64::read(heap, args)?;
        Self::from_unix_timestamp(secs).map_err(|_| ProtocolError::from("cannot read datetime"))
    }
}

#[cfg(feature = "time")]
impl Incoming for time::Date {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, days) = i32::init(args)?;
        let d = Self::from_julian_day(days);
        match d {
            Ok(d) => Ok((0, d)),
            Err(_) => Err(ProtocolError(TIME_PARSE_ERROR)),
        }
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let days = self.to_julian_day();
        days.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "time")]
impl Outcoming for time::Date {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let days = self.to_julian_day();
        days.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let days = i32::read(heap, args)?;
        Self::from_julian_day(days).map_err(|_| ProtocolError::from("cannot read datetime"))
    }
}

/**
convert u32 to time::Time
*/
#[cfg(feature = "time")]
pub const fn time_from_u32(u: u32) -> Result<time::Time, time::error::ComponentRange> {
    let bytes: [u8; 4] = u.to_le_bytes();
    let hour = bytes[0];
    let minute = bytes[1];
    let second = bytes[2];
    time::Time::from_hms(hour, minute, second)
}

/**
convert time::Time to u32
*/
#[cfg(feature = "time")]
pub const fn time_into_u32(time: &time::Time) -> u32 {
    let hour = time.hour();
    let minute = time.minute();
    let second = time.second();
    u32::from_le_bytes([hour, minute, second, 0])
}

#[cfg(feature = "time")]
impl Incoming for time::Time {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, u) = u32::init(args)?;
        let time = time_from_u32(u).map_err(|_| ProtocolError(TIME_PARSE_ERROR))?;
        Ok((0, time))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let u = time_into_u32(self);
        u.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "time")]
impl Outcoming for time::Time {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let u = time_into_u32(self);
        u.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let u = u32::read(heap, args)?;
        let time = time_from_u32(u)?;
        Ok(time)
    }
}

// TODO: used?
#[derive(PartialEq, Clone, Default)]
pub struct Bytes(Vec<u8>);

// TODO: for wasm64 other logic
impl Incoming for Bytes {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let mut vec: Vec<u8> = Vec::with_capacity(len);

        for _ in 0..quot {
            let u = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            for byte in &bytes {
                vec.push(*byte);
            }
        }

        if !is_divided {
            let u = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            let mut iter = bytes.iter();
            for _ in 0..rem {
                let byte = *iter.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
                vec.push(byte);
            }
        }

        Ok((0, Bytes(vec)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.0.len();
        args.push(len as u32);

        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let count = if is_divided { quot } else { quot + 1 };
        let mut vec: Vec<u32> = Vec::with_capacity(count);
        let mut iter = self.0.iter();

        for _ in 0..quot {
            let bytes: [u8; 4] = [
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
            ];

            let u = u32::from_le_bytes(bytes);
            vec.push(u);
        }

        if !is_divided {
            let b1 = *iter
                .next()
                .ok_or_else(|| ProtocolError("args is end".to_string()))?;
            let b2 = *iter.next().unwrap_or(&0);
            let b3 = *iter.next().unwrap_or(&0);
            let b4 = *iter.next().unwrap_or(&0);
            let u = u32::from_le_bytes([b1, b2, b3, b4]);
            vec.push(u);
        }

        args.append(&mut vec);

        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?; // len
        let len = self.0.len();
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let count = if is_divided { quot } else { quot + 1 };

        for _ in 0..count {
            args.next()
                .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        }
        Ok(())
    }
}

// TODO: for wasm64 other logic
impl Outcoming for Bytes {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.0.len();
        args.push(len as u32);

        // TODO: for wasm64 other logic
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let count = if is_divided { quot } else { quot + 1 };
        let mut vec: Vec<u32> = Vec::with_capacity(count);
        let mut iter = self.0.iter();

        for _ in 0..quot {
            let bytes: [u8; 4] = [
                *iter.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?,
                *iter.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?,
                *iter.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?,
                *iter.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?,
            ];

            let u = u32::from_le_bytes(bytes);
            vec.push(u);
        }

        if !is_divided {
            let b1 = *iter.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
            let b2 = *iter.next().unwrap_or(&0);
            let b3 = *iter.next().unwrap_or(&0);
            let b4 = *iter.next().unwrap_or(&0);
            let u = u32::from_le_bytes([b1, b2, b3, b4]);
            vec.push(u);
        }

        args.append(&mut vec);

        Ok(())
    }

    // TODO: https://stackoverflow.com/questions/49690459/converting-a-vecu32-to-vecu8-in-place-and-with-minimal-overhead
    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let len = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as usize;
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let mut vec: Vec<u8> = Vec::with_capacity(len);

        for _ in 0..quot {
            let u = *args
                .next()
                .ok_or_else(|| ProtocolError("args is end".to_string()))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            for byte in &bytes {
                vec.push(*byte);
            }
        }

        if !is_divided {
            let u = *args
                .next()
                .ok_or_else(|| ProtocolError("args is end".to_string()))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            let mut iter = bytes.iter();
            for _ in 0..rem {
                let byte = *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?;
                vec.push(byte);
            }
        }

        Ok(Bytes(vec))
    }
}

/*impl Incoming for String {
    const IS_NEED_INIT_FILL: bool = true;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        // узнаем длину строки
        let arg = args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let len = *arg as usize;
        // создаем массив байт указанной длины состоящий из нулевых байт
        let vec = vec![0u8; len];
        // let s = String::with_capacity(len);
        let string = unsafe { String::from_utf8_unchecked(vec) };
        let ptr = string.as_ptr() as u32;
        *arg = ptr;
        Ok((ptr, string)) // why ptr?
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(self.len() as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        let ptr: usize = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?
            as usize; // its pointer to string
        let mut pointer = ptr;
        for byte in self.as_bytes() {
            heap[pointer] = *byte;
            pointer += 1;
        }
        Ok(())
    }
}*/

/*impl Outcoming for String {
    const IS_NEED_READ: bool = true;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(self.len() as u32);
        args.push(self.as_ptr() as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let len = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as usize;
        let ptr = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as usize;
        let bytes = &heap[ptr..ptr + len];
        Ok(String::from_utf8(bytes.to_vec())?)
    }
}*/

impl Incoming for String {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let mut vec: Vec<u8> = Vec::with_capacity(len);

        for _ in 0..quot {
            let u = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            for byte in &bytes {
                vec.push(*byte);
            }
        }

        if !is_divided {
            let u = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            let mut iter = bytes.iter();
            for _ in 0..rem {
                let byte = *iter.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
                vec.push(byte);
            }
        }

        let s = unsafe { String::from_utf8_unchecked(vec) };

        Ok((0, s))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len();
        args.push(len as u32);

        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let count = if is_divided { quot } else { quot + 1 };
        let mut vec: Vec<u32> = Vec::with_capacity(count);
        let mut iter = self.as_bytes().iter();

        for _ in 0..quot {
            let bytes: [u8; 4] = [
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
                *iter
                    .next()
                    .ok_or_else(|| ProtocolError("args is end".to_string()))?,
            ];

            let u = u32::from_le_bytes(bytes);
            vec.push(u);
        }

        if !is_divided {
            let b1 = *iter
                .next()
                .ok_or_else(|| ProtocolError("args is end".to_string()))?;
            let b2 = *iter.next().unwrap_or(&0);
            let b3 = *iter.next().unwrap_or(&0);
            let b4 = *iter.next().unwrap_or(&0);
            let u = u32::from_le_bytes([b1, b2, b3, b4]);
            vec.push(u);
        }

        args.append(&mut vec);

        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?; // len
        let len = self.len();
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let count = if is_divided { quot } else { quot + 1 };

        for _ in 0..count {
            args.next()
                .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        }
        Ok(())
    }
}

impl Outcoming for String {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes = self.as_bytes().to_vec();
        let bytes = Bytes(bytes);
        bytes.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let bytes = Bytes::read(heap, args)?;
        let bytes = bytes.0;
        let s = String::from_utf8(bytes);
        s.map_err(|e| e.into())
    }
}

// TODO: other realization for bytes vec
impl<T: Incoming> Incoming for Vec<T> {
    const IS_NEED_INIT_FILL: bool = T::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let arg = args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let len = *arg as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            let item: T = T::init(args)?.1;
            vec.push(item);
        }
        // TODO: ptr for not destructing?
        let ptr = vec.as_mut_ptr() as u32;
        *arg = ptr; // TODO: not need?
        Ok((ptr, vec))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len();
        args.push(len as u32);
        for item in self {
            item.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?; // len
        for item in self {
            item.fill(heap, args)?;
        }
        Ok(())
    }
}

impl<T: Outcoming> Outcoming for Vec<T> {
    const IS_NEED_READ: bool = T::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len() as u32;
        args.push(len);
        for item in self {
            item.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let len = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as usize;
        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            let item: T = T::read(heap, args)?;
            vec.push(item);
        }
        Ok(vec)
    }
}

impl<T: Incoming> Incoming for Option<T> {
    const IS_NEED_INIT_FILL: bool = T::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let is_some: bool = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? != 0;

        if !is_some {
            Ok((0, None))
        } else {
            // TODO: ???
            let (ptr, item) = T::init(args)?;
            Ok((ptr, Some(item)))
        }
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        match self {
            None => {
                args.push(0);
            }
            Some(item) => {
                args.push(1);
                item.args(args)?;
            }
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        if let Some(item) = self {
            item.fill(heap, args)?;
        }
        Ok(())
    }
}

impl<T: Outcoming> Outcoming for Option<T> {
    const IS_NEED_READ: bool = T::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        match self {
            None => {
                args.push(0);
            }
            Some(item) => {
                args.push(1);
                item.args(args)?;
            }
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let is_some = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?
            != 0;

        if !is_some {
            Ok(None)
        } else {
            Ok(Some(T::read(heap, args)?))
        }
    }
}

#[cfg(any(feature = "std", feature = "hashmap"))]
impl<K: Incoming, V: Incoming> Incoming for HashMap<K, V>
where
    K: Eq + Hash,
{
    const IS_NEED_INIT_FILL: bool = K::IS_NEED_INIT_FILL || V::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let mut map: HashMap<K, V> = HashMap::with_capacity(len);
        for _ in 0..len {
            let (key_ptr, key) = K::init(args)?;
            let (value_ptr, value) = V::init(args)?;
            map.insert(key, value)
                .ok_or(ProtocolError(MAP_INSERT_ERROR))?;
        }
        Ok((0, map))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len();
        args.push(len as u32);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?; // len
        for (key, value) in self {
            key.fill(heap, args)?;
            value.fill(heap, args)?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "std", feature = "hashmap"))]
impl<K: Outcoming, V: Outcoming> Outcoming for HashMap<K, V>
where
    K: Eq + Hash,
{
    const IS_NEED_READ: bool = K::IS_NEED_READ || V::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len() as u32;
        args.push(len);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let len = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as usize;
        let mut map: HashMap<K, V> = HashMap::with_capacity(len);
        for _ in 0..len {
            let key: K = K::read(heap, args)?;
            let value: V = V::read(heap, args)?;
            map.insert(key, value)
                .ok_or_else(|| ProtocolError("map already have item".to_string()))?;
        }
        Ok(map)
    }
}

impl<K: Incoming, V: Incoming> Incoming for BTreeMap<K, V>
where
    K: Ord,
{
    const IS_NEED_INIT_FILL: bool = K::IS_NEED_INIT_FILL || V::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let mut map: BTreeMap<K, V> = BTreeMap::new();
        for _ in 0..len {
            let (_, key) = K::init(args)?;
            let (_, value) = V::init(args)?;
            map.insert(key, value)
                .ok_or(ProtocolError(MAP_INSERT_ERROR))?;
        }
        Ok((0, map))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len();
        args.push(len as u32);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?; // len
        for (key, value) in self {
            key.fill(heap, args)?;
            value.fill(heap, args)?;
        }
        Ok(())
    }
}

impl<K: Outcoming, V: Outcoming> Outcoming for BTreeMap<K, V>
where
    K: Ord,
{
    const IS_NEED_READ: bool = K::IS_NEED_READ || V::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len() as u32;
        args.push(len);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let len = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as usize;
        let mut map: BTreeMap<K, V> = BTreeMap::new();
        for _ in 0..len {
            let key: K = K::read(heap, args)?;
            let value: V = V::read(heap, args)?;
            map.insert(key, value)
                .ok_or_else(|| ProtocolError("map already have item".to_string()))?;
        }
        Ok(map)
    }
}

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
impl<K: Incoming, V: Incoming> Incoming for FxHashMap<K, V>
where
    K: Eq + Hash,
{
    const IS_NEED_INIT_FILL: bool = K::IS_NEED_INIT_FILL || V::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let mut map: FxHashMap<K, V> =
            FxHashMap::with_capacity_and_hasher(len, FxBuildHasher::default());
        for _ in 0..len {
            let (key_ptr, key) = K::init(args)?;
            let (value_ptr, value) = V::init(args)?;
            map.insert(key, value)
                .ok_or(ProtocolError(MAP_INSERT_ERROR))?;
        }
        Ok((0, map))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len();
        args.push(len as u32);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        args.next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))?;
        for (key, value) in self {
            key.fill(heap, args)?;
            value.fill(heap, args)?;
        }
        Ok(())
    }
}

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
impl<K: Outcoming, V: Outcoming> Outcoming for FxHashMap<K, V>
where
    K: Eq + Hash,
{
    const IS_NEED_READ: bool = K::IS_NEED_READ || V::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let len = self.len() as u32;
        args.push(len);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let len = *args
            .next()
            .ok_or_else(|| ProtocolError("args is end".to_string()))? as usize;
        let mut map: FxHashMap<K, V> =
            FxHashMap::with_capacity_and_hasher(len, FxBuildHasher::default());
        for _ in 0..len {
            let key: K = K::read(heap, args)?;
            let value: V = V::read(heap, args)?;
            map.insert(key, value)
                .ok_or_else(|| ProtocolError("map already have item".to_string()))?;
        }
        Ok(map)
    }
}

impl<T1: Incoming, T2: Incoming> Incoming for (T1, T2) {
    const IS_NEED_INIT_FILL: bool = T1::IS_NEED_INIT_FILL || T2::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t1) = T1::init(args)?;
        let (_, t2) = T2::init(args)?;
        Ok((0, (t1, t2)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        self.0.fill(heap, args)?;
        self.1.fill(heap, args)?;
        Ok(())
    }
}

impl<T1: Outcoming, T2: Outcoming> Outcoming for (T1, T2) {
    const IS_NEED_READ: bool = T1::IS_NEED_READ || T2::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let t1 = T1::read(heap, args)?;
        let t2 = T2::read(heap, args)?;
        Ok((t1, t2))
    }
}

impl<T1: Incoming, T2: Incoming, T3: Incoming> Incoming for (T1, T2, T3) {
    const IS_NEED_INIT_FILL: bool =
        T1::IS_NEED_INIT_FILL || T2::IS_NEED_INIT_FILL || T3::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t1) = T1::init(args)?;
        let (_, t2) = T2::init(args)?;
        let (_, t3) = T3::init(args)?;
        Ok((0, (t1, t2, t3)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        self.0.fill(heap, args)?;
        self.1.fill(heap, args)?;
        self.2.fill(heap, args)?;
        Ok(())
    }
}

impl<T1: Outcoming, T2: Outcoming, T3: Outcoming> Outcoming for (T1, T2, T3) {
    const IS_NEED_READ: bool = T1::IS_NEED_READ || T2::IS_NEED_READ || T3::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let t1 = T1::read(heap, args)?;
        let t2 = T2::read(heap, args)?;
        let t3 = T3::read(heap, args)?;
        Ok((t1, t2, t3))
    }
}

impl<T1: Incoming, T2: Incoming, T3: Incoming, T4: Incoming> Incoming for (T1, T2, T3, T4) {
    const IS_NEED_INIT_FILL: bool = T1::IS_NEED_INIT_FILL
        || T2::IS_NEED_INIT_FILL
        || T3::IS_NEED_INIT_FILL
        || T4::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t1) = T1::init(args)?;
        let (_, t2) = T2::init(args)?;
        let (_, t3) = T3::init(args)?;
        let (_, t4) = T4::init(args)?;
        Ok((0, (t1, t2, t3, t4)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        self.3.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        self.0.fill(heap, args)?;
        self.1.fill(heap, args)?;
        self.2.fill(heap, args)?;
        self.3.fill(heap, args)?;
        Ok(())
    }
}

impl<T1: Outcoming, T2: Outcoming, T3: Outcoming, T4: Outcoming> Outcoming for (T1, T2, T3, T4) {
    const IS_NEED_READ: bool =
        T1::IS_NEED_READ || T2::IS_NEED_READ || T3::IS_NEED_READ || T4::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        self.3.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let t1 = T1::read(heap, args)?;
        let t2 = T2::read(heap, args)?;
        let t3 = T3::read(heap, args)?;
        let t4 = T4::read(heap, args)?;
        Ok((t1, t2, t3, t4))
    }
}

impl<T: Incoming> Incoming for Box<T> {
    const IS_NEED_INIT_FILL: bool = T::IS_NEED_INIT_FILL;

    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t) = T::init(args)?;
        Ok((0, Box::new(t)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.as_ref().args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), ProtocolError> {
        self.as_ref().fill(heap, args)?;
        Ok(())
    }
}

impl<T: Outcoming> Outcoming for Box<T> {
    const IS_NEED_READ: bool = T::IS_NEED_READ;

    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.as_ref().args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, ProtocolError> {
        let t = T::read(heap, args)?;
        Ok(Box::new(t))
    }
}

// TODO: impl Incoming and Outcoming for HashSet
