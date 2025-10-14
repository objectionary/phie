// SPDX-FileCopyrightText: Copyright (c) 2023 Eugene Darashkevich
// SPDX-License-Identifier: MIT

extern crate phie;

use phie::data::Data;
use phie::emu::{Emu, Opt};
use std::env;
use std::fs;
use std::str::FromStr;

fn emulate(phi_code: &str) -> Data {
    let mut emu: Emu = Emu::from_str(phi_code).unwrap_or_else(|e| {
        eprintln!("Failed to parse phi code: {}", e);
        std::process::exit(1);
    });
    emu.opt(Opt::LogSnapshots);
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    emu.dataize().0
}

pub fn run_emulator(filename: &str) -> i16 {
    let binding = fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Failed to read file '{}': {}", filename, e);
        std::process::exit(1);
    });
    let phi_code: &str = binding.as_str();
    emulate(phi_code)
}

pub fn execute_program(args: &[String]) -> i16 {
    assert!(args.len() >= 2);
    let filename: &str = &args[1];
    let result: i16 = run_emulator(filename);
    if args.len() >= 3 {
        let correct = args[2].parse::<i16>().unwrap_or_else(|e| {
            eprintln!("Invalid expected value argument '{}': {}", args[2], e);
            std::process::exit(1);
        });
        assert_eq!(result, correct);
    }
    result
}

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2);
    let result = execute_program(&args);
    println!("Executor result: {}", result);
}

#[test]
#[should_panic]
fn test_main() {
    main();
}

#[test]
fn test_execute_program_with_valid_args() {
    let args = vec![
        "program_name".to_string(),
        "tests/resources/written_test_example".to_string(),
        "84".to_string(),
    ];
    let result = execute_program(&args);
    assert_eq!(result, 84);
}

#[test]
#[should_panic]
fn test_execute_program_with_invalid_args() {
    let args = vec!["program_name".to_string()];
    execute_program(&args);
}

#[test]
fn executes_file_example() {
    assert_eq!(84, run_emulator("tests/resources/written_test_example"));
}

#[test]
fn executes_fibonacci_file() {
    assert_eq!(21, run_emulator("tests/resources/written_fibonacci_test"));
}

#[test]
fn executes_sum_file() {
    assert_eq!(84, run_emulator("tests/resources/written_sum_test"));
}
