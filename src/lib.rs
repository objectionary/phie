// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

#![deny(warnings)]

pub mod atom;
pub mod basket;
pub mod data;
pub mod emu;
pub mod loc;
pub mod locator;
pub mod object;
pub mod perf;

#[cfg(test)]
use simple_logger::SimpleLogger;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    SimpleLogger::new().init().expect("Failed to initialize logger in tests");
}
