// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

extern crate phie;

use phie::data::Data;
use phie::emu::{Emu, Opt};
use std::env;

pub fn fibo(x: Data) -> Data {
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
    .expect("Failed to parse Fibonacci emulator");
    emu.opt(Opt::LogSnapshots);
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    emu.dataize().0
}

pub fn parse_fibonacci_args(args: &[String]) -> Result<(Data, i32), String> {
    if args.len() < 3 {
        return Err(format!(
            "Usage: {} <input> <cycles>",
            args.first().unwrap_or(&"fibonacci".to_string())
        ));
    }
    let input = args[1]
        .parse()
        .map_err(|e| format!("Invalid input argument '{}': {}", args[1], e))?;
    let cycles = args[2]
        .parse()
        .map_err(|e| format!("Invalid cycles argument '{}': {}", args[2], e))?;
    Ok((input, cycles))
}

pub fn run_fibonacci_cycles(input: Data, cycles: i32) -> (Data, Data) {
    let mut total = 0;
    let mut f = 0;
    for _ in 0..cycles {
        f = fibo(input);
        total += f;
    }
    (f, total)
}

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let (input, cycles) = match parse_fibonacci_args(&args) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    let (f, total) = run_fibonacci_cycles(input, cycles);
    println!("{}-th Fibonacci number is {}", input, f);
    println!("Sum of results is {}", total);
}

#[cfg(test)]
use simple_logger::SimpleLogger;

#[test]
fn calculates_fibonacci() {
    SimpleLogger::new()
        .init()
        .expect("Failed to init logger in test");
    assert_eq!(21, fibo(7))
}

#[test]
fn calculates_fibonacci_for_multiple_inputs() {
    assert_eq!(13, fibo(6));
    assert_eq!(34, fibo(8));
    assert_eq!(55, fibo(9));
    assert_eq!(89, fibo(10));
}

#[test]
fn calculates_fibonacci_five() {
    assert_eq!(8, fibo(5));
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
    let args = vec![
        "fibonacci".to_string(),
        "invalid".to_string(),
        "3".to_string(),
    ];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid input argument"));
}

#[test]
fn fails_to_parse_invalid_cycles() {
    let args = vec![
        "fibonacci".to_string(),
        "7".to_string(),
        "invalid".to_string(),
    ];
    let result = parse_fibonacci_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid cycles argument"));
}

#[test]
fn runs_fibonacci_single_cycle() {
    let (f, total) = run_fibonacci_cycles(5, 1);
    assert_eq!(f, 8);
    assert_eq!(total, 8);
}

#[test]
fn runs_fibonacci_multiple_cycles() {
    let (f, total) = run_fibonacci_cycles(7, 3);
    assert_eq!(f, 21);
    assert_eq!(total, 63);
}

#[test]
fn runs_fibonacci_zero_cycles() {
    let (f, total) = run_fibonacci_cycles(7, 0);
    assert_eq!(f, 0);
    assert_eq!(total, 0);
}
