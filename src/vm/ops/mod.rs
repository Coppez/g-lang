//! Opcode handler implementations for the VM.
//!
//! Split into logical groups:
//! - `stack_vars` — stack manipulation, locals, globals, builtins, jumps
//! - `arithmetic` — numeric ops (macros), comparisons, not, negate, truthiness
//! - `collections` — arrays, hashes, indexing
//! - `structs` — struct building, field access/mutation, method calls
//! - `calls` — function invocation, closures, async, await
//! - `exceptions` — throw, catch, finally
//! - `modules` — import, export

pub mod arithmetic;
pub mod collections;
pub mod exceptions;
pub mod stack_vars;
pub mod structs;
pub mod calls;
pub mod modules;
