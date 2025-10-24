// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

extern crate phie;

use phie::data::Data;
use phie::emu::{Emu, Opt};
use std::env;

pub fn fibo(x: Data) -> Result<Data, String> {
    let mut emu: Emu = format!(
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x{:04X} ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ), 𝛼0 ↦ ν1(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν13(𝜋) ⟧
        ν5(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧
        ν6(𝜋) ↦ ⟦ λ ↦ int-sub, ρ ↦ 𝜋.𝜋.𝛼0, 𝛼0 ↦ ν5(𝜋) ⟧
        ν7(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν8(𝜋) ↦ ⟦ λ ↦ int-sub, ρ ↦ 𝜋.𝜋.𝛼0, 𝛼0 ↦ ν7(𝜋) ⟧
        ν9(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ), 𝛼0 ↦ ν8(𝜋) ⟧
        ν10(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ), 𝛼0 ↦ ν6(𝜋) ⟧
        ν11(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν9(𝜋), 𝛼0 ↦ ν10(𝜋) ⟧
        ν12(𝜋) ↦ ⟦ λ ↦ int-less, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ ν5(𝜋) ⟧
        ν13(𝜋) ↦ ⟦ λ ↦ bool-if, ρ ↦ ν12(𝜋), 𝛼0 ↦ ν7(𝜋), 𝛼1 ↦ ν11(𝜋) ⟧
        ",
        x
    )
    .parse()
    .map_err(|e| format!("Failed to parse Fibonacci emulator: {}", e))?;
    emu.opt(Opt::LogSnapshots);
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    Ok(emu.dataize().0)
}

pub fn parse_fibonacci_args(args: &[String]) -> Result<(Data, i32), String> {
    if args.len() < 3 {
        return Err(format!(
            "Usage: {} <input> <cycles>",
            args.first().unwrap_or(&"fibonacci".to_string())
        ));
    }
    let input =
        args[1].parse().map_err(|e| format!("Invalid input argument '{}': {}", args[1], e))?;
    let cycles =
        args[2].parse().map_err(|e| format!("Invalid cycles argument '{}': {}", args[2], e))?;
    Ok((input, cycles))
}

pub fn run_fibonacci_cycles(input: Data, cycles: i32) -> Result<(Data, Data), String> {
    let mut total = 0;
    let mut f = 0;
    for _ in 0..cycles {
        f = fibo(input)?;
        total += f;
    }
    Ok((f, total))
}

pub fn run(args: &[String]) -> Result<String, String> {
    let (input, cycles) = parse_fibonacci_args(args)?;
    let (f, total) = run_fibonacci_cycles(input, cycles)?;
    Ok(format!("{}-th Fibonacci number is {}\nSum of results is {}", input, f, total))
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

#[cfg(test)]
use simple_logger::SimpleLogger;

#[test]
fn calculates_fibonacci() {
    SimpleLogger::new().init().expect("Failed to init logger in test");
    assert_eq!(21, fibo(7).expect("Failed to calculate fibonacci"))
}

#[test]
fn calculates_fibonacci_for_multiple_inputs() {
    assert_eq!(13, fibo(6).expect("Failed to calculate fibonacci"));
    assert_eq!(34, fibo(8).expect("Failed to calculate fibonacci"));
    assert_eq!(55, fibo(9).expect("Failed to calculate fibonacci"));
    assert_eq!(89, fibo(10).expect("Failed to calculate fibonacci"));
}

#[test]
fn calculates_fibonacci_five() {
    assert_eq!(8, fibo(5).expect("Failed to calculate fibonacci"));
}

#[test]
fn parses_valid_fibonacci_args() {
    let args = vec!["fibonacci".to_string(), "7".to_string(), "3".to_string()];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_ok());
    let (input, cycles) = result.unwrap();
    assert_eq!(input, 7);
    assert_eq!(cycles, 3);
}

#[test]
fn fails_to_parse_insufficient_args() {
    let args = vec!["fibonacci".to_string(), "7".to_string()];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn fails_to_parse_invalid_input() {
    let args = vec!["fibonacci".to_string(), "invalid".to_string(), "3".to_string()];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid input argument"));
}

#[test]
fn fails_to_parse_invalid_cycles() {
    let args = vec!["fibonacci".to_string(), "7".to_string(), "invalid".to_string()];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid cycles argument"));
}

#[test]
fn runs_fibonacci_single_cycle() {
    let (f, total) = run_fibonacci_cycles(5, 1).expect("Failed to run fibonacci cycles");
    assert_eq!(f, 8);
    assert_eq!(total, 8);
}

#[test]
fn runs_fibonacci_multiple_cycles() {
    let (f, total) = run_fibonacci_cycles(7, 3).expect("Failed to run fibonacci cycles");
    assert_eq!(f, 21);
    assert_eq!(total, 63);
}

#[test]
fn runs_fibonacci_zero_cycles() {
    let (f, total) = run_fibonacci_cycles(7, 0).expect("Failed to run fibonacci cycles");
    assert_eq!(f, 0);
    assert_eq!(total, 0);
}

#[test]
fn test_run_success() {
    let args = vec!["fibonacci".to_string(), "5".to_string(), "3".to_string()];
    let result = run(&args);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("5-th Fibonacci number is 8"));
    assert!(output.contains("Sum of results is 24"));
}

#[test]
fn test_run_with_insufficient_args() {
    let args = vec!["fibonacci".to_string(), "5".to_string()];
    let result = run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn test_run_with_invalid_input() {
    let args = vec![
        "fibonacci".to_string(),
        "invalid".to_string(),
        "3".to_string(),
    ];
    let result = run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid input argument"));
}

#[test]
fn test_parse_with_empty_args() {
    let args: Vec<String> = vec![];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn test_run_with_empty_args() {
    let args: Vec<String> = vec![];
    let result = run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn test_parse_args_with_single_arg() {
    let args = vec!["fibonacci".to_string()];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("fibonacci"));
}

#[test]
fn test_run_with_invalid_cycles_format() {
    let args = vec![
        "fibonacci".to_string(),
        "5".to_string(),
        "invalid".to_string(),
    ];
    let result = run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid cycles argument"));
}

#[test]
fn runs_fibonacci_with_negative_cycles() {
    let (f, total) = run_fibonacci_cycles(5, -1).expect("Failed to run fibonacci cycles");
    assert_eq!(f, 0);
    assert_eq!(total, 0);
}

#[test]
fn calculates_fibonacci_edge_cases() {
    assert_eq!(1, fibo(0).expect("Failed to calculate fibonacci"));
    assert_eq!(1, fibo(1).expect("Failed to calculate fibonacci"));
    assert_eq!(2, fibo(2).expect("Failed to calculate fibonacci"));
    assert_eq!(3, fibo(3).expect("Failed to calculate fibonacci"));
}

#[test]
fn test_run_with_two_args() {
    let args = vec!["fibonacci".to_string(), "5".to_string()];
    let result = run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn runs_fibonacci_multiple_iterations() {
    let (f, total) = run_fibonacci_cycles(6, 5).expect("Failed to run fibonacci cycles");
    assert_eq!(f, 13);
    assert_eq!(total, 65);
}

#[test]
fn test_parse_args_with_zero_input() {
    let args = vec!["fibonacci".to_string(), "0".to_string(), "1".to_string()];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_ok());
    let (input, cycles) = result.unwrap();
    assert_eq!(input, 0);
    assert_eq!(cycles, 1);
}

#[test]
fn test_run_success_with_different_values() {
    let args = vec!["fibonacci".to_string(), "6".to_string(), "2".to_string()];
    let result = run(&args);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("6-th Fibonacci number is 13"));
    assert!(output.contains("Sum of results is 26"));
}

#[test]
fn calculates_fibonacci_11() {
    assert_eq!(144, fibo(11).expect("Failed to calculate fibonacci"));
}

#[test]
fn calculates_fibonacci_4() {
    assert_eq!(5, fibo(4).expect("Failed to calculate fibonacci"));
}

#[test]
fn runs_fibonacci_with_large_cycles() {
    let (f, total) = run_fibonacci_cycles(4, 10).expect("Failed to run fibonacci cycles");
    assert_eq!(f, 5);
    assert_eq!(total, 50);
}

#[test]
fn test_parse_args_with_negative_input() {
    let args = vec!["fibonacci".to_string(), "-5".to_string(), "1".to_string()];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_ok());
}

#[test]
fn test_run_with_zero_input() {
    let args = vec!["fibonacci".to_string(), "0".to_string(), "1".to_string()];
    let result = run(&args);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("0-th Fibonacci number is 1"));
}

#[test]
fn test_run_with_large_input() {
    let args = vec!["fibonacci".to_string(), "12".to_string(), "1".to_string()];
    let result = run(&args);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("12-th Fibonacci number is 233"));
}

#[test]
fn runs_fibonacci_with_exactly_one_cycle() {
    let (f, total) = run_fibonacci_cycles(10, 1).expect("Failed to run fibonacci cycles");
    assert_eq!(f, 89);
    assert_eq!(total, 89);
}

#[test]
fn calculates_fibonacci_12() {
    assert_eq!(233, fibo(12).expect("Failed to calculate fibonacci"));
}

#[test]
fn calculates_fibonacci_2() {
    assert_eq!(2, fibo(2).expect("Failed to calculate fibonacci"));
}
