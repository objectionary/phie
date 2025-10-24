// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Command-line interface for phie emulator.
//!
//! This module provides functionality to parse command-line arguments,
//! read phie program files, and execute them using the Emu emulator.
//!
//! # Example
//!
//! ```
//! use phie::cli;
//!
//! let args = vec!["phie".to_string(), "program.phie".to_string()];
//! match cli::run(&args) {
//!     Ok(output) => println!("{}", output),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

use std::{fs, path::Path};

use crate::data::Data;
use crate::emu::{Emu, Opt};

/// Parses command line arguments and extracts the file path.
///
/// Validates that exactly one file path argument is provided.
///
/// # Arguments
///
/// * `args` - Slice of command line arguments where first element is program name
///
/// # Returns
///
/// * `Ok(String)` - File path to process
/// * `Err(String)` - Error message with usage information
///
/// # Examples
///
/// ```
/// use phie::cli::parse_args;
///
/// let args = vec!["phie".to_string(), "test.phie".to_string()];
/// let result = parse_args(&args);
/// assert_eq!(result.unwrap(), "test.phie");
/// ```
pub fn parse_args(args: &[String]) -> Result<String, String> {
    if args.len() < 2 {
        return Err(format!(
            "Usage: {} <file.phie>",
            args.first().map(|s| s.as_str()).unwrap_or("phie")
        ));
    }
    Ok(args[1].clone())
}

/// Reads and returns content from a phie program file.
///
/// Validates file existence before reading and provides detailed error messages.
///
/// # Arguments
///
/// * `file_path` - Path to the phie program file
///
/// # Returns
///
/// * `Ok(String)` - File content
/// * `Err(String)` - Error message if file doesn't exist or cannot be read
///
/// # Examples
///
/// ```no_run
/// use phie::cli::read_phie_file;
///
/// let content = read_phie_file("program.phie").unwrap();
/// ```
pub fn read_phie_file(file_path: &str) -> Result<String, String> {
    if !Path::new(file_path).exists() {
        return Err(format!("File '{}' does not exist", file_path));
    }
    fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))
}

/// Executes a phie program and returns the dataized result.
///
/// Parses the program content into an Emu instance, configures execution options,
/// and performs dataization to compute the result.
///
/// # Arguments
///
/// * `content` - Program content in phie calculus notation
///
/// # Returns
///
/// * `Ok(Data)` - Computed result after dataization
/// * `Err(String)` - Error message if parsing or execution fails
///
/// # Examples
///
/// ```
/// use phie::cli::execute_phie;
///
/// let program = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧";
/// let result = execute_phie(program).unwrap();
/// assert_eq!(result, 42);
/// ```
pub fn execute_phie(content: &str) -> Result<Data, String> {
    let mut emu: Emu =
        content.parse().map_err(|e| format!("Failed to parse phie program: {}", e))?;
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    Ok(emu.dataize().0)
}

/// Main execution pipeline for the CLI.
///
/// Orchestrates argument parsing, file reading, and program execution.
///
/// # Arguments
///
/// * `args` - Command line arguments including program name
///
/// # Returns
///
/// * `Ok(String)` - Formatted execution result
/// * `Err(String)` - Error message from any pipeline stage
///
/// # Examples
///
/// ```no_run
/// use phie::cli::run;
///
/// let args = vec!["phie".to_string(), "program.phie".to_string()];
/// match run(&args) {
///     Ok(output) => println!("{}", output),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn run(args: &[String]) -> Result<String, String> {
    let file_path = parse_args(args)?;
    let content = read_phie_file(&file_path)?;
    let result = execute_phie(&content)?;
    Ok(format!("{}", result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_args() {
        let args = vec!["phie".to_string(), "test.phie".to_string()];
        let result = parse_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test.phie");
    }

    #[test]
    fn fails_to_parse_insufficient_args() {
        let args = vec!["phie".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn fails_to_parse_empty_args() {
        let args: Vec<String> = vec![];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn parse_args_returns_first_file() {
        let args = vec!["phie".to_string(), "first.phie".to_string(), "second.phie".to_string()];
        let result = parse_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "first.phie");
    }

    #[test]
    fn parse_args_preserves_path() {
        let args = vec!["phie".to_string(), "/path/to/file.phie".to_string()];
        let result = parse_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/path/to/file.phie");
    }

    #[test]
    fn fails_to_read_nonexistent_file() {
        let result = read_phie_file("/tmp/nonexistent_phie_file_xyz123.phie");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn executes_simple_data_program() {
        let program = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧";
        let result = execute_phie(program);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn executes_addition_program() {
        let program = "
            ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν3(𝜋) ⟧
            ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
            ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ 𝜋.𝛼1 ⟧
            ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼0 ↦ ν1(𝜋), 𝛼1 ↦ ν1(𝜋) ⟧
        ";
        let result = execute_phie(program);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 84);
    }

    #[test]
    fn fails_to_execute_invalid_syntax() {
        let program = "invalid program syntax";
        let result = execute_phie(program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse"));
    }

    #[test]
    fn executes_phi_reference() {
        let program = "
            ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
            ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ";
        let result = execute_phie(program);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn executes_hex_value() {
        let program = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x00FF ⟧";
        let result = execute_phie(program);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 255);
    }

    #[test]
    fn executes_zero_value() {
        let program = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0000 ⟧";
        let result = execute_phie(program);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn executes_single_byte_value() {
        let program = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧";
        let result = execute_phie(program);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
