// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use phie::cli;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static MKTEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn mktemp(filename: &str) -> (PathBuf, String) {
    let serial = MKTEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let mut file = std::env::temp_dir();
    file.push(format!("phie-{pid}-{serial}-{filename}"));
    let path = file.clone().into_os_string().into_string().unwrap();
    (file, path)
}

#[test]
fn runs_simple_program_from_file() {
    let (file, path) = mktemp("phie_test_simple.phie");
    fs::write(&file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧").unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn runs_addition_program() {
    let (file, path) = mktemp("phie_test_addition.phie");
    let program = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν3(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ 𝜋.𝛼1 ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼0 ↦ ν1(𝜋), 𝛼1 ↦ ν1(𝜋) ⟧
    ";
    fs::write(&file, program).unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "84");
}

#[test]
fn fails_with_nonexistent_file() {
    let (_, path) = mktemp("nonexistent_xyz.phie");
    let args = vec!["phie".to_string(), path];
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
    let (file, path) = mktemp("phie_test_invalid.phie");
    fs::write(&file, "invalid syntax").unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse"));
}

#[test]
fn reads_multiline_program() {
    let (file, path) = mktemp("phie_test_multiline.phie");
    let program = "ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧\nν1(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧";
    fs::write(&file, program).unwrap();
    let result = cli::read_phie_file(&path);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("ν0"));
    assert!(content.contains("ν1"));
}

#[test]
fn handles_whitespace_in_file() {
    let (file, path) = mktemp("phie_test_whitespace.phie");
    let content = "  \n  ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧  \n  ";
    fs::write(&file, content).unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn executes_large_hex_value() {
    let (file, path) = mktemp("phie_test_hex.phie");
    fs::write(&file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x00FF ⟧").unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "255");
}

#[test]
fn executes_zero_value() {
    let (file, path) = mktemp("phie_test_zero.phie");
    fs::write(&file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0000 ⟧").unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn executes_one_value() {
    let (file, path) = mktemp("phie_test_one.phie");
    fs::write(&file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧").unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1");
}

#[test]
fn executes_hundred_value() {
    let (file, path) = mktemp("phie_test_hundred.phie");
    fs::write(&file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x0064 ⟧").unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "100");
}

#[test]
fn handles_phi_reference() {
    let (file, path) = mktemp("phie_test_phi_ref.phie");
    let program = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
    ";
    fs::write(&file, program).unwrap();
    let args = vec!["phie".to_string(), path];
    let result = cli::run(&args);
    fs::remove_file(file).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[cfg(unix)]
#[test]
fn fails_with_unreadable_file() {
    use std::{fs::Permissions, os::unix::fs::PermissionsExt};
    let (file, path) = mktemp("phie_test_unreadable.phie");
    fs::write(&file, "content").unwrap();
    fs::set_permissions(&file, Permissions::from_mode(0o000)).unwrap();
    let result = cli::read_phie_file(&path);
    fs::set_permissions(&file, Permissions::from_mode(0o644)).unwrap();
    fs::remove_file(file).unwrap();
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

#[test]
fn mktemp_returns_unique_paths_per_call() {
    use std::collections::HashSet;
    let mut paths: HashSet<String> = HashSet::new();
    for _ in 0..16 {
        let (_, path) = mktemp("phie_test_unique.phie");
        assert!(
            paths.insert(path.clone()),
            "mktemp returned a duplicate path on repeated calls: {}",
            path
        );
    }
}

#[test]
fn parallel_mktemp_users_do_not_clash() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    let collected: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    for _ in 0..16 {
        let collected = collected.clone();
        let errors = errors.clone();
        handles.push(thread::spawn(move || {
            let (file, path) = mktemp("phie_test_parallel.phie");
            collected.lock().unwrap().push(path.clone());
            if let Err(err) = fs::write(&file, "ν0(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧") {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("write {}: {}", path, err));
                return;
            }
            let result = cli::run(&["phie".to_string(), path.clone()]);
            if let Err(err) = fs::remove_file(&file) {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("remove {}: {}", path, err));
            }
            if !matches!(result.as_deref(), Ok("42")) {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("run {} returned {:?}", path, result));
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let errors = errors.lock().unwrap();
    assert!(
        errors.is_empty(),
        "parallel mktemp users clashed: {:?}",
        *errors
    );
    let collected = collected.lock().unwrap();
    let unique: std::collections::HashSet<&String> = collected.iter().collect();
    assert_eq!(
        collected.len(),
        unique.len(),
        "mktemp returned duplicate paths under parallel use: {:?}",
        *collected
    );
}

#[test]
fn prints_usage_when_help_is_requested() {
    let args = vec!["phie".to_string(), "--help".to_string()];
    let result = cli::run(&args);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("Usage: phie <file.phie>"));
}

#[test]
fn prints_usage_when_help_is_requested_briefly() {
    let args = vec!["phie".to_string(), "-h".to_string()];
    let result = cli::run(&args);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert!(result.unwrap().contains("Usage: phie <file.phie>"));
}
