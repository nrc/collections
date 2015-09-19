#![feature(heap_api, core_intrinsics, unique, optin_builtin_traits, alloc)]

extern crate alloc;
extern crate crossbeam;

pub mod stack;
pub mod treiber;
pub mod hashmap;
