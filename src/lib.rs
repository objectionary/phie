// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

#![deny(warnings)]

pub mod atom;
pub mod basket;
pub mod cli;
pub mod data;
pub mod emu;
pub mod loc;
pub mod locator;
pub mod object;
pub mod perf;
pub mod rust_atom;
pub mod rust_engine;
pub mod universe;

#[cfg(test)]
use simple_logger::SimpleLogger;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    let _ = SimpleLogger::new().init();
}
