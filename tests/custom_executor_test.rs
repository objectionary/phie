// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use assert_cmd::Command;

#[test]
fn call_custom_executor() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.arg("tests/resources/written_test_example")
        .assert()
        .success()
        .stdout("Executor result: 84\n");
}

#[test]
fn fails_with_no_args() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.assert().failure().stderr(predicates::str::contains("Usage:"));
}

#[test]
fn call_executor_with_sum_file() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.arg("tests/resources/written_sum_test").assert().success().stdout("Executor result: 84\n");
}

#[test]
fn call_executor_with_fibonacci_file() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.arg("tests/resources/written_fibonacci_test")
        .assert()
        .success()
        .stdout("Executor result: 21\n");
}

#[test]
fn call_executor_with_expected_value_match() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.arg("tests/resources/written_test_example")
        .arg("84")
        .assert()
        .success()
        .stdout("Executor result: 84\n");
}

#[test]
fn fails_with_expected_value_mismatch() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.arg("tests/resources/written_test_example")
        .arg("100")
        .assert()
        .failure()
        .stderr(predicates::str::contains("does not match expected"));
}

#[test]
fn fails_with_invalid_file() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.arg("nonexistent_file.txt")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Failed to read file"));
}

#[test]
fn fails_with_invalid_expected_value() {
    let mut cmd = Command::cargo_bin("custom_executor").unwrap();
    cmd.arg("tests/resources/written_test_example")
        .arg("not_a_number")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid expected value argument"));
}
