// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use assert_cmd::Command;

#[test]
fn calculates_fibo() {
    let mut cmd = Command::cargo_bin("fibonacci").unwrap();
    cmd.arg("5")
        .arg("3")
        .assert()
        .success()
        .stdout("5-th Fibonacci number is 8\nSum of results is 24\n");
}

#[test]
fn fails_with_insufficient_args() {
    let mut cmd = Command::cargo_bin("fibonacci").unwrap();
    cmd.arg("5").assert().failure().stderr(predicates::str::contains("Usage:"));
}

#[test]
fn fails_with_invalid_input() {
    let mut cmd = Command::cargo_bin("fibonacci").unwrap();
    cmd.arg("invalid")
        .arg("3")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid input argument"));
}

#[test]
fn fails_with_invalid_cycles() {
    let mut cmd = Command::cargo_bin("fibonacci").unwrap();
    cmd.arg("5")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid cycles argument"));
}

#[test]
fn calculates_with_zero() {
    let mut cmd = Command::cargo_bin("fibonacci").unwrap();
    cmd.arg("0")
        .arg("1")
        .assert()
        .success()
        .stdout(predicates::str::contains("0-th Fibonacci number is 1"));
}

#[test]
fn calculates_with_large_number() {
    let mut cmd = Command::cargo_bin("fibonacci").unwrap();
    cmd.arg("10")
        .arg("1")
        .assert()
        .success()
        .stdout(predicates::str::contains("10-th Fibonacci number is 89"));
}

#[test]
fn fails_with_no_args() {
    let mut cmd = Command::cargo_bin("fibonacci").unwrap();
    cmd.assert().failure().stderr(predicates::str::contains("Usage:"));
}
