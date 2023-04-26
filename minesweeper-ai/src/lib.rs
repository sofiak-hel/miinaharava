//! This module simply represents the library part of this crate, mostly used by
//! benches, tests and other toolings

#![deny(clippy::all)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
// lib.rs exists mostly for bench etc, dead_code is actually handled seperately
// in lib.rs and main.rs, so dead_code is still disallowed through main.rs
#![allow(dead_code)]

pub mod ai;
mod thread_controller;
