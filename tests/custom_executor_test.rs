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
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Usage:"));
}
