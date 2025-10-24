// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use std::fs;

use phie::cli;

#[test]
fn runs_simple_program_from_file() {
    let temp_file = "/tmp/phie_test_simple.phie";
    fs::write(temp_file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧").unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn runs_addition_program() {
    let temp_file = "/tmp/phie_test_addition.phie";
    let program = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν3(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ 𝜋.𝛼1 ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼0 ↦ ν1(𝜋), 𝛼1 ↦ ν1(𝜋) ⟧
    ";
    fs::write(temp_file, program).unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "84");
}

#[test]
fn fails_with_nonexistent_file() {
    let args = vec!["phie".to_string(), "/tmp/nonexistent_xyz.phie".to_string()];
    let result = cli::run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn fails_with_no_arguments() {
    let args = vec!["phie".to_string()];
    let result = cli::run(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn fails_with_invalid_program() {
    let temp_file = "/tmp/phie_test_invalid.phie";
    fs::write(temp_file, "invalid syntax").unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse"));
}

#[test]
fn reads_multiline_program() {
    let temp_file = "/tmp/phie_test_multiline.phie";
    let program = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧\nν1(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧";
    fs::write(temp_file, program).unwrap();
    let result = cli::read_phie_file(temp_file);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("ν0"));
    assert!(content.contains("ν1"));
}

#[test]
fn handles_whitespace_in_file() {
    let temp_file = "/tmp/phie_test_whitespace.phie";
    let content = "  \n  ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧  \n  ";
    fs::write(temp_file, content).unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn executes_large_hex_value() {
    let temp_file = "/tmp/phie_test_hex.phie";
    fs::write(temp_file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x00FF ⟧").unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "255");
}

#[test]
fn executes_zero_value() {
    let temp_file = "/tmp/phie_test_zero.phie";
    fs::write(temp_file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0000 ⟧").unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn executes_one_value() {
    let temp_file = "/tmp/phie_test_one.phie";
    fs::write(temp_file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧").unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1");
}

#[test]
fn executes_hundred_value() {
    let temp_file = "/tmp/phie_test_hundred.phie";
    fs::write(temp_file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0064 ⟧").unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "100");
}

#[test]
fn handles_phi_reference() {
    let temp_file = "/tmp/phie_test_phi_ref.phie";
    let program = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
    ";
    fs::write(temp_file, program).unwrap();
    let args = vec!["phie".to_string(), temp_file.to_string()];
    let result = cli::run(&args);
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[cfg(unix)]
#[test]
fn fails_with_unreadable_file() {
    use std::{fs::Permissions, os::unix::fs::PermissionsExt};
    let temp_file = "/tmp/phie_test_unreadable.phie";
    fs::write(temp_file, "content").unwrap();
    fs::set_permissions(temp_file, Permissions::from_mode(0o000)).unwrap();
    let result = cli::read_phie_file(temp_file);
    fs::set_permissions(temp_file, Permissions::from_mode(0o644)).unwrap();
    fs::remove_file(temp_file).unwrap();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read file"));
}

#[test]
fn preserves_file_path_with_special_chars() {
    let args = vec!["phie".to_string(), "test-file_123.phie".to_string()];
    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-file_123.phie");
}

#[test]
fn preserves_absolute_path() {
    let args = vec!["phie".to_string(), "/absolute/path/test.phie".to_string()];
    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/absolute/path/test.phie");
}
