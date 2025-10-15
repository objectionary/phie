// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2023 Eugene Darashkevich
// SPDX-License-Identifier: MIT

extern crate phie;

use phie::data::Data;
use phie::emu::{Emu, Opt};
use std::env;
use std::fs;
use std::str::FromStr;

fn emulate(phi_code: &str) -> Result<Data, String> {
    let mut emu: Emu =
        Emu::from_str(phi_code).map_err(|e| format!("Failed to parse phi code: {}", e))?;
    emu.opt(Opt::LogSnapshots);
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    Ok(emu.dataize().0)
}

pub fn run_emulator(filename: &str) -> Result<i16, String> {
    let binding = fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file '{}': {}", filename, e))?;
    let phi_code: &str = binding.as_str();
    emulate(phi_code)
}

pub fn execute_program(args: &[String]) -> Result<i16, String> {
    if args.len() < 2 {
        return Err("Insufficient arguments".to_string());
    }
    let filename: &str = &args[1];
    let result: i16 = run_emulator(filename)?;
    if args.len() >= 3 {
        let correct = args[2]
            .parse::<i16>()
            .map_err(|e| format!("Invalid expected value argument '{}': {}", args[2], e))?;
        if result != correct {
            return Err(format!(
                "Result {} does not match expected {}",
                result, correct
            ));
        }
    }
    Ok(result)
}

pub fn validate_and_execute(args: &[String]) -> Result<i16, String> {
    if args.len() < 2 {
        return Err(format!(
            "Usage: {} <filename> [expected_result]",
            args.first().unwrap_or(&"custom_executor".to_string())
        ));
    }
    execute_program(args)
}

pub fn run(args: &[String]) -> Result<String, String> {
    let result = validate_and_execute(args)?;
    Ok(format!("Executor result: {}", result))
}

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    match run(&args) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

#[test]
fn test_execute_program_with_valid_args() {
    let args = vec![
        "program_name".to_string(),
        "tests/resources/written_test_example".to_string(),
        "84".to_string(),
    ];
    let result = execute_program(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 84);
}

#[test]
fn test_execute_program_with_invalid_args() {
    let args = vec!["program_name".to_string()];
    let result = execute_program(&args);
    assert!(result.is_err());
}

#[test]
fn executes_file_example() {
    assert_eq!(
        84,
        run_emulator("tests/resources/written_test_example").unwrap()
    );
}

#[test]
fn executes_fibonacci_file() {
    assert_eq!(
        21,
        run_emulator("tests/resources/written_fibonacci_test").unwrap()
    );
}

#[test]
fn executes_sum_file() {
    assert_eq!(
        84,
        run_emulator("tests/resources/written_sum_test").unwrap()
    );
}

#[test]
fn test_emulate_basic() {
    let phi_code = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
    ";
    assert_eq!(42, emulate(phi_code).unwrap());
}

#[test]
fn test_execute_program_single_arg() {
    let args = vec![
        "program_name".to_string(),
        "tests/resources/written_test_example".to_string(),
    ];
    let result = execute_program(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 84);
}

#[test]
fn test_run_emulator_sum() {
    let result = run_emulator("tests/resources/written_sum_test");
    assert_eq!(result.unwrap(), 84);
}

#[test]
fn test_emulate_simple_data() {
    let phi_code = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧";
    assert_eq!(1, emulate(phi_code).unwrap());
}

#[test]
fn test_emulate_with_lambda() {
    let phi_code = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ Δ ↦ 0x0005 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0003 ⟧
    ";
    assert_eq!(8, emulate(phi_code).unwrap());
}

#[test]
fn test_run_emulator_fibonacci() {
    let result = run_emulator("tests/resources/written_fibonacci_test");
    assert_eq!(21, result.unwrap());
}

#[test]
fn validates_and_executes_with_valid_args() {
    let args = vec![
        "custom_executor".to_string(),
        "tests/resources/written_test_example".to_string(),
    ];
    let result = validate_and_execute(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 84);
}

#[test]
fn validates_and_executes_with_expected_result() {
    let args = vec![
        "custom_executor".to_string(),
        "tests/resources/written_sum_test".to_string(),
        "84".to_string(),
    ];
    let result = validate_and_execute(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 84);
}

#[test]
fn fails_validation_with_insufficient_args() {
    let args = vec!["custom_executor".to_string()];
    let result = validate_and_execute(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn validates_with_empty_args() {
    let args: Vec<String> = vec![];
    let result = validate_and_execute(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn test_run_success() {
    let args = vec![
        "custom_executor".to_string(),
        "tests/resources/written_test_example".to_string(),
    ];
    let result = run(&args);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Executor result: 84"));
}

#[test]
fn test_run_with_no_args() {
    let args: Vec<String> = vec![];
    let result = run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn test_run_with_single_arg() {
    let args = vec!["custom_executor".to_string()];
    let result = run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn test_emulate_with_invalid_phi_code() {
    let result = emulate("invalid phi code");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Failed to parse phi code"));
}

#[test]
fn test_run_emulator_with_nonexistent_file() {
    let result = run_emulator("nonexistent_file.txt");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Failed to read file"));
}

#[test]
fn test_execute_program_with_invalid_expected_value() {
    let args = vec![
        "program".to_string(),
        "tests/resources/written_test_example".to_string(),
        "not_a_number".to_string(),
    ];
    let result = execute_program(&args);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Invalid expected value argument"));
}

#[test]
fn test_execute_program_with_wrong_expected_value() {
    let args = vec![
        "program".to_string(),
        "tests/resources/written_test_example".to_string(),
        "999".to_string(),
    ];
    let result = execute_program(&args);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("does not match expected"));
}

#[test]
fn test_execute_program_insufficient_args() {
    let args = vec!["program".to_string()];
    let result = execute_program(&args);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Insufficient arguments"));
}

#[test]
fn test_emulate_with_multiple_operations() {
    let phi_code = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ λ ↦ int-times, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ Δ ↦ 0x0006 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0007 ⟧
    ";
    assert_eq!(42, emulate(phi_code).unwrap());
}

#[test]
fn test_run_with_valid_file() {
    let args = vec![
        "custom_executor".to_string(),
        "tests/resources/written_sum_test".to_string(),
    ];
    let result = run(&args);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Executor result: 84"));
}

#[test]
fn test_validate_and_execute_with_one_arg() {
    let args = vec!["custom_executor".to_string()];
    let result = validate_and_execute(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("custom_executor"));
}

#[test]
fn test_execute_program_with_correct_expected() {
    let args = vec![
        "program".to_string(),
        "tests/resources/written_fibonacci_test".to_string(),
        "21".to_string(),
    ];
    let result = execute_program(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 21);
}

#[test]
fn test_run_emulator_fibonacci_file() {
    let result = run_emulator("tests/resources/written_fibonacci_test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 21);
}

#[test]
fn test_emulate_negative_number() {
    let phi_code = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ λ ↦ int-neg, ρ ↦ ν2(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ Δ ↦ 0x0005 ⟧
    ";
    assert_eq!(-5, emulate(phi_code).unwrap());
}
