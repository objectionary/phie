// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Command-line tool for executing phie calculus programs.
//!
//! This binary provides a simple interface to run phie programs from files.
//!
//! # Usage
//!
//! ```bash
//! phie program.phie
//! ```
//!
//! The program file should contain phie calculus expressions in the format:
//! ```text
//! ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
//! ```

extern crate phie;

use std::{env::args, process::exit};

use env_logger::init as logger_init;

use phie::cli::run;

fn main() {
    logger_init();
    let args: Vec<String> = args().collect();
    match run(&args) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
