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
