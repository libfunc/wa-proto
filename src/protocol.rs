//! Зачем разделили на Incoming и Outcoming?
//! Конечно чаще всего структуры будут имплементировать оба трейта.
//! Но иногда возможно какую то структуру будут передавать в wasm,
//! но из wasm она не будет передаваться никогда. Тогда для этой
//! структуры излишне имплементировать Outcoming.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, boxed::Box,collections::BTreeMap};
#[cfg(not(feature = "std"))]
use core::{convert::{TryInto}, slice::IterMut};
#[cfg(all(not(feature = "std"), feature = "hashmap"))]
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::{
    cell::RefMut,
    convert::{TryInto},
    slice::Iter,
    string::String,
    vec::Vec,
    collections::{HashMap, BTreeMap},
    hash::Hash,
    error::Error,
};
#[cfg(feature = "chrono")]
use chrono::{Date, DateTime, Duration, Utc, NaiveDateTime, NaiveDate, Datelike};
#[cfg(feature = "std")]
use core::fmt;
#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
use crate::{FxHashMap, FxBuildHasher};

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
pub const ENUM_FROM_U32_ERROR: u32 = 7;
pub const TIME_PARSE_ERROR: u32 = 8;

#[cfg(feature = "std")]
impl From<&str> for ProtocolError {
    fn from(s: &str) -> Self {
        ProtocolError(s.to_string())
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
    Инициализируем кусок памяти в wasm для последующего заполнения.
    Вызывается в wasm.
    */
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> where Self: Sized;

    /**
    Добавление в аргументы вспомогательных данных,
    таких как длина строки/массива и др.
    Вызывается на хосте.
    */
    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>>;

    /**
    Заполнение данными инициализированный участок памяти.
    вызывается на хосте.
    Иногда инициализация не нужна, если значение можно полностью передать в аргументах,
    уместить в значении u32.
    */
    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>>;

    /**
    Функиця указывает на то что необходимо инициализировать даные, а затем заполнить
    если true - получается 2 шага
    если false - 1 шаг
    */
    fn is_need_init_fill() -> bool { true }
}

/**
Outcoming trait (Serializable) - (Исходящее сообщение) если структура реализует этот трейт,
то эту структуру можно передать из wasm на хост.

Будет в основном использоваться для передачи сообщений между wasm модулем и Rust рантаймом.
Для сериализации и десериализации в БД рекомендуется использовать serde.
*/
pub trait Outcoming {
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> where Self: Sized;

    /**
    Функиця указывает на то что необходимо прочитать данные из памяти песочницы
    */
    fn is_need_read() -> bool { true }
}

impl Incoming for bool {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el != 0))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        args.push(if *self { 1 } else { 0 });
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

impl Outcoming for bool {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(if *self { 1 } else { 0 });
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(el != 0)
    }

    fn is_need_read() -> bool { false }
}

impl Incoming for u8 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el as u8))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

impl Outcoming for u8 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(el as u8)
    }

    fn is_need_read() -> bool { false }
}

impl Incoming for i32 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el as i32))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

impl Outcoming for i32 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        Ok(*args.next().ok_or(ProtocolError("args is end".to_string()))? as i32)
    }

    fn is_need_read() -> bool { false }
}

impl Incoming for i64 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el1: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let el2: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR8_ERROR))?;
        let e = i64::from_le_bytes(*d);
        Ok((0, e))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
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
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

impl Outcoming for i64 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arr2: &[u8; 4] = bytes[4..8].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el1: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        let el2: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into()?;
        let e = i64::from_le_bytes(*d);
        Ok(e)
    }

    fn is_need_read() -> bool { false }
}

impl Incoming for u32 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((el, el))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        args.push(*self);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

impl Outcoming for u32 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        Ok(*args.next().ok_or(ProtocolError("args is end".to_string()))?)
    }

    fn is_need_read() -> bool { false }
}

impl Incoming for u64 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el1: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let el2: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR8_ERROR))?;
        let e = u64::from_le_bytes(*d);
        Ok((0, e))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
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
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

impl Outcoming for u64 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arr2: &[u8; 4] = bytes[4..8].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el1: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        let el2: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into()?;
        let e = u64::from_le_bytes(*d);
        Ok(e)
    }

    fn is_need_read() -> bool { false }
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
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
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(el as usize)
    }

    fn is_need_read() -> bool { false }
}

// only for wasm32 and runner target_pointer_width = "32"
impl Incoming for isize {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        Ok((0, el as isize))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

// only for wasm32 and runner target_pointer_width = "32"
impl Outcoming for isize {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(*self as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(el as isize)
    }

    fn is_need_read() -> bool { false }
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let bytes = self.to_le_bytes();
        let u = u32::from_le_bytes(bytes);
        args.push(u);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
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
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        let bytes: [u8; 4] = el.to_le_bytes();
        let f = f32::from_le_bytes(bytes);
        Ok(f)
    }

    fn is_need_read() -> bool { false }
}

impl Incoming for f64 {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let el1: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let el2: u32 = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR8_ERROR))?;
        let e = f64::from_le_bytes(*d);
        Ok((0, e))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
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
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool { false }
}

impl Outcoming for f64 {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let bytes: [u8; 8] = self.to_le_bytes();
        let arr1: &[u8; 4] = bytes[0..4].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arr2: &[u8; 4] = bytes[4..8].try_into().map_err(|_| ProtocolError(BYTES_INTO_ARR4_ERROR))?;
        let arg1: u32 = u32::from_le_bytes(*arr1);
        let arg2: u32 = u32::from_le_bytes(*arr2);
        args.push(arg1);
        args.push(arg2);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let el1: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        let el2: u32 = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
        let b1: [u8; 4] = el1.to_le_bytes();
        let b2: [u8; 4] = el2.to_be_bytes();
        let c: Vec<u8> = [&b1[..], &b2[..]].concat();
        let d: &[u8; 8] = &c[..].try_into()?;
        let e = f64::from_le_bytes(*d);
        Ok(e)
    }

    fn is_need_read() -> bool { false }
}

impl Incoming for String {
    // TODO: don't copy bytes, read in place (построить структуру на месте)?
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        // узнаем длину строки
        let arg = args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))?;
        let len = *arg as usize;
        // создаем массив байт указанной длины состоящий из нулевых байт
        let vec = vec![0u8; len];
        // тут будет ошибка? нулевые байты не валидная utf-8 строка
        // нет, так как метод from_utf8 проверяет на валидность, но страдает производительность
        // let s = String::with_capacity(len); // TODO: ?
        // TODO: from_utf8_unchecked?
        let string = String::from_utf8(vec).map_err(|_| ProtocolError(STRING_FROM_BYTES_ERROR))?;
        let ptr = string.as_ptr() as u32;
        *arg = ptr;
        Ok((ptr, string)) // why ptr?
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        args.push(self.len() as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        let ptr: usize = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize; // its pointer to string
        let mut pointer = ptr;
        for byte in self.as_bytes() {
            heap[pointer] = *byte;
            pointer += 1;
        }
        Ok(())
    }
}

impl Outcoming for String {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        args.push(self.len() as u32);
        args.push(self.as_ptr() as u32);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>)  -> Result<Self, Box<dyn Error>> {
        let len = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize;
        let ptr = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize;
        let bytes = &heap[ptr..ptr + len];
        // TODO: or from_utf8_unchecked ?
        Ok(String::from_utf8(bytes.to_vec())?)
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let e = self.num_milliseconds();
        e.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let e = i64::read(heap, args)?;
        Ok(Duration::milliseconds(e))
    }

    fn is_need_read() -> bool {
        false
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let secs = self.timestamp();
        secs.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let secs = i64::read(heap, args)?;
        Ok(Self::from_utc(NaiveDateTime::from_timestamp(secs, 0), Utc))
    }

    fn is_need_read() -> bool {
        false
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let days = self.num_days_from_ce();
        days.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let days = i32::read(heap, args)?;
        Ok(Self::from_utc(NaiveDate::from_num_days_from_ce(days), Utc))
    }

    fn is_need_read() -> bool {
        false
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let e = self.whole_seconds();
        e.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let e = i64::read(heap, args)?;
        Ok(time::Duration::seconds(e))
    }

    fn is_need_read() -> bool {
        false
    }
}

#[cfg(feature = "time")]
impl Incoming for time::OffsetDateTime {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, secs) = i64::init(args)?;
        let dt = Self::from_unix_timestamp(secs);
        Ok((0, dt))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let secs = self.unix_timestamp();
        secs.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let secs = i64::read(heap, args)?;
        Ok(Self::from_unix_timestamp(secs))
    }

    fn is_need_read() -> bool {
        false
    }
}

#[cfg(feature = "time")]
impl Incoming for time::Date {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, days) = i64::init(args)?;
        let d = Self::from_julian_day(days);
        Ok((0, d))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let days = self.julian_day();
        days.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
    }
}

#[cfg(feature = "time")]
impl Outcoming for time::Date {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        let days = self.julian_day();
        days.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let days = i64::read(heap, args)?;
        Ok(Self::from_julian_day(days))
    }

    fn is_need_read() -> bool {
        false
    }
}

#[cfg(feature = "time")]
pub const fn time_from_u32(u: u32) -> Result<time::Time, time::ComponentRangeError> {
    let bytes: [u8; 4] = u.to_le_bytes();
    let hour = bytes[0];
    let minute = bytes[1];
    let second = bytes[2];
    time::Time::try_from_hms(hour, minute, second)
}

#[cfg(feature = "time")]
pub const fn time_into_u32(time: &time::Time) -> u32 {
    let hour = time.hour();
    let minute = time.minute();
    let second = time.second();
    let u = u32::from_le_bytes([hour, minute, second,  0]);
    u
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let u = time_into_u32(self);
        u.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let u = u32::read(heap, args)?;
        let time = time_from_u32(u)?;
        Ok(time)
    }

    fn is_need_read() -> bool {
        false
    }
}

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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
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
                *iter.next().ok_or(ProtocolError("args is end".to_string()))?,
                *iter.next().ok_or(ProtocolError("args is end".to_string()))?,
                *iter.next().ok_or(ProtocolError("args is end".to_string()))?,
                *iter.next().ok_or(ProtocolError("args is end".to_string()))?,
            ];

            let u = u32::from_le_bytes(bytes);
            vec.push(u);
        }

        if !is_divided {
            let b1 = *iter.next().ok_or(ProtocolError("args is end".to_string()))?;
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
    fn fill(&self, _: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?; // len
        let len = self.0.len();
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let count = if is_divided { quot } else { quot + 1 };

        for _ in 0..count {
            args.next().ok_or(ProtocolError("args is end".to_string()))?;
        }
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        false
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
    fn read(_: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let len = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize;
        let quot = len / 4;
        let rem = len % 4;
        let is_divided = rem == 0;
        let mut vec: Vec<u8> = Vec::with_capacity(len);

        for _ in 0..quot {
            let u = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            for byte in &bytes {
                vec.push(*byte);
            }
        }

        if !is_divided {
            let u = *args.next().ok_or(ProtocolError("args is end".to_string()))?;
            let bytes: [u8; 4] = u.to_le_bytes();
            let mut iter = bytes.iter();
            for _ in 0..rem {
                let byte = *iter.next().ok_or(ProtocolError("args is end".to_string()))?;
                vec.push(byte);
            }
        }

        Ok(Bytes(vec))
    }

    fn is_need_read() -> bool {
        false
    }
}

impl<T: Incoming> Incoming for Vec<T> {
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let len = self.len();
        args.push(len as u32);
        for item in self {
            item.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?; // len
        for item in self {
            item.fill(heap, args)?;
        }
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        T::is_need_init_fill()
    }
}

impl<T: Outcoming> Outcoming for Vec<T> {
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let len = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize;
        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            let item: T = T::read(heap, args)?;
            vec.push(item);
        }
        Ok(vec)
    }

    fn is_need_read() -> bool {
        T::is_need_read()
    }
}

impl<T: Incoming> Incoming for Option<T> {
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
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
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
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        if let Some(item) = self {
            item.fill(heap, args)?;
        }
        Ok(())
    }

    fn is_need_init_fill() -> bool { T::is_need_init_fill() }
}

impl<T: Outcoming> Outcoming for Option<T> {
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let is_some = *args.next().ok_or(ProtocolError("args is end".to_string()))? != 0;

        if !is_some {
            Ok(None)
        } else {
            Ok(Some(T::read(heap, args)?))
        }
    }

    fn is_need_read() -> bool { T::is_need_read() }
}

#[cfg(any(feature = "std", feature = "hashmap"))]
impl<K: Incoming, V: Incoming> Incoming for HashMap<K, V>
    where K: Eq + Hash {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let mut map: HashMap<K, V> = HashMap::with_capacity(len);
        for _ in 0..len {
            let (key_ptr, key) = K::init(args)?;
            let (value_ptr, value) = V::init(args)?;
            map.insert(key, value).ok_or(ProtocolError(MAP_INSERT_ERROR))?;
        }
        Ok((0, map))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let len = self.len();
        args.push(len as u32);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?; // len
        for (key, value) in self {
            key.fill(heap, args)?;
            value.fill(heap, args)?;
        }
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        K::is_need_init_fill() || V::is_need_init_fill()
    }
}

#[cfg(any(feature = "std", feature = "hashmap"))]
impl<K: Outcoming, V: Outcoming> Outcoming for HashMap<K, V>
    where K: Eq + Hash {
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let len = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize;
        let mut map: HashMap<K, V> = HashMap::with_capacity(len);
        for _ in 0..len {
            let key: K = K::read(heap, args)?;
            let value: V = V::read(heap, args)?;
            map.insert(key, value).ok_or(ProtocolError("map already have item".to_string()))?;
        }
        Ok(map)
    }

    fn is_need_read() -> bool {
        K::is_need_read() || V::is_need_read()
    }
}

impl<K: Incoming, V: Incoming> Incoming for BTreeMap<K, V>
    where K: Ord {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let mut map: BTreeMap<K, V> = BTreeMap::new();
        for _ in 0..len {
            let (_, key) = K::init(args)?;
            let (_, value) = V::init(args)?;
            map.insert(key, value).ok_or(ProtocolError(MAP_INSERT_ERROR))?;
        }
        Ok((0, map))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let len = self.len();
        args.push(len as u32);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?; // len
        for (key, value) in self {
            key.fill(heap, args)?;
            value.fill(heap, args)?;
        }
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        K::is_need_init_fill() || V::is_need_init_fill()
    }
}

impl<K: Outcoming, V: Outcoming> Outcoming for BTreeMap<K, V>
    where K: Ord {
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let len = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize;
        let mut map: BTreeMap<K, V> = BTreeMap::new();
        for _ in 0..len {
            let key: K = K::read(heap, args)?;
            let value: V = V::read(heap, args)?;
            map.insert(key, value).ok_or(ProtocolError("map already have item".to_string()))?;
        }
        Ok(map)
    }

    fn is_need_read() -> bool {
        K::is_need_read() || V::is_need_read()
    }
}

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
impl<K: Incoming, V: Incoming> Incoming for FxHashMap<K, V>
    where K: Eq + Hash {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let len = *args.next().ok_or(ProtocolError(ARGS_NEXT_ERROR))? as usize;
        let mut map: FxHashMap<K, V> = FxHashMap::with_capacity_and_hasher(len, FxBuildHasher::default());
        for _ in 0..len {
            let (key_ptr, key) = K::init(args)?;
            let (value_ptr, value) = V::init(args)?;
            map.insert(key, value).ok_or(ProtocolError(MAP_INSERT_ERROR))?;
        }
        Ok((0, map))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        let len = self.len();
        args.push(len as u32);
        for (key, value) in self {
            key.args(args)?;
            value.args(args)?;
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        args.next().ok_or(ProtocolError("args is end".to_string()))?;
        for (key, value) in self {
            key.fill(heap, args)?;
            value.fill(heap, args)?;
        }
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        K::is_need_init_fill() || V::is_need_init_fill()
    }
}

#[cfg(all(any(feature = "hashmap", feature = "std"), feature = "map"))]
impl<K: Outcoming, V: Outcoming> Outcoming for FxHashMap<K, V>
    where K: Eq + Hash {
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
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let len = *args.next().ok_or(ProtocolError("args is end".to_string()))? as usize;
        let mut map: FxHashMap<K, V> = FxHashMap::with_capacity_and_hasher(len, FxBuildHasher::default());
        for _ in 0..len {
            let key: K = K::read(heap, args)?;
            let value: V = V::read(heap, args)?;
            map.insert(key, value).ok_or(ProtocolError("map already have item".to_string()))?;
        }
        Ok(map)
    }

    fn is_need_read() -> bool {
        K::is_need_read() || V::is_need_read()
    }
}

impl<T1: Incoming, T2: Incoming> Incoming for (T1, T2) {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t1) = T1::init(args)?;
        let (_, t2) = T2::init(args)?;
        Ok((0, (t1, t2)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        self.0.args(args)?;
        self.1.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        self.0.fill(heap, args)?;
        self.1.fill(heap, args)?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        T1::is_need_init_fill() || T2::is_need_init_fill()
    }
}

impl<T1: Outcoming, T2: Outcoming> Outcoming for (T1, T2) {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let t1 = T1::read(heap, args)?;
        let t2 = T2::read(heap, args)?;
        Ok((t1, t2))
    }

    fn is_need_read() -> bool {
        T1::is_need_read() || T2::is_need_read()
    }
}

impl<T1: Incoming, T2: Incoming, T3: Incoming> Incoming for (T1, T2, T3) {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t1) = T1::init(args)?;
        let (_, t2) = T2::init(args)?;
        let (_, t3) = T3::init(args)?;
        Ok((0, (t1, t2, t3)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        self.0.fill(heap, args)?;
        self.1.fill(heap, args)?;
        self.2.fill(heap, args)?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        T1::is_need_init_fill() || T2::is_need_init_fill() || T3::is_need_init_fill()
    }
}

impl<T1: Outcoming, T2: Outcoming, T3: Outcoming> Outcoming for (T1, T2, T3) {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let t1 = T1::read(heap, args)?;
        let t2 = T2::read(heap, args)?;
        let t3 = T3::read(heap, args)?;
        Ok((t1, t2, t3))
    }

    fn is_need_read() -> bool {
        T1::is_need_read() || T2::is_need_read() || T3::is_need_read()
    }
}

impl<T1: Incoming, T2: Incoming, T3: Incoming, T4: Incoming> Incoming for (T1, T2, T3, T4) {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t1) = T1::init(args)?;
        let (_, t2) = T2::init(args)?;
        let (_, t3) = T3::init(args)?;
        let (_, t4) = T4::init(args)?;
        Ok((0, (t1, t2, t3, t4)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        self.3.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        self.0.fill(heap, args)?;
        self.1.fill(heap, args)?;
        self.2.fill(heap, args)?;
        self.3.fill(heap, args)?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        T1::is_need_init_fill()
            || T2::is_need_init_fill()
            || T3::is_need_init_fill()
            || T4::is_need_init_fill()
    }
}

impl<T1: Outcoming, T2: Outcoming, T3: Outcoming, T4: Outcoming> Outcoming for (T1, T2, T3, T4) {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.0.args(args)?;
        self.1.args(args)?;
        self.2.args(args)?;
        self.3.args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let t1 = T1::read(heap, args)?;
        let t2 = T2::read(heap, args)?;
        let t3 = T3::read(heap, args)?;
        let t4 = T4::read(heap, args)?;
        Ok((t1, t2, t3, t4))
    }

    fn is_need_read() -> bool {
        T1::is_need_read()
            || T2::is_need_read()
            || T3::is_need_read()
            || T4::is_need_read()
    }
}

impl<T: Incoming> Incoming for Box<T> {
    #[cfg(not(feature = "std"))]
    fn init(args: &mut IterMut<u32>) -> Result<(u32, Self), ProtocolError> {
        let (_, t) = T::init(args)?;
        Ok((0, Box::new(t)))
    }

    #[cfg(feature = "std")]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
        self.as_ref().args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn fill(&self, heap: &mut RefMut<[u8]>, args: &mut Iter<u32>) -> Result<(), Box<dyn Error>> {
        self.as_ref().fill(heap, args)?;
        Ok(())
    }

    fn is_need_init_fill() -> bool {
        T::is_need_init_fill()
    }
}

impl<T: Outcoming> Outcoming for Box<T> {
    #[cfg(not(feature = "std"))]
    fn args(&self, args: &mut Vec<u32>) -> Result<(), ProtocolError> {
        self.as_ref().args(args)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn read(heap: &[u8], args: &mut Iter<u32>) -> Result<Self, Box<dyn Error>> {
        let t = T::read(heap, args)?;
        Ok(Box::new(t))
    }

    fn is_need_read() -> bool {
        T::is_need_read()
    }
}

// TODO: impl Incoming and Outcoming for HashSet
