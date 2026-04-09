//! Tree-walk interpreter for G-lang.
//!
//! # Modules
//!
//! - `eval` — the main [`Evaluator`] that walks the AST and produces [`Object`]s
//! - `env` — scoped variable environments with O(1) slot-based lookups
//! - `obj` — the [`Object`] enum representing all runtime values
//! - `builtins` — standard library functions (string, math, io, http, etc.)
//! - `module_registry` — module loading, caching, and WASM integration
//! - `constant_pool` — compile-time literal extraction for faster evaluation
//! - `helpers` — shared evaluation utilities (type converters, sync eval helpers)

pub mod env;
pub mod eval;
pub mod obj;
pub mod builtins;
pub mod module_registry;
pub mod helpers;
pub mod constant_pool;