//! This module simply represents the library part of this crate, mostly used by
//! benches, tests and other toolings

#![deny(clippy::all)]
#![allow(dead_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub mod ai;
mod thread_controller;
