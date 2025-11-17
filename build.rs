// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let test_libs_dir = out_dir.join("test_libs");
    fs::create_dir_all(&test_libs_dir).unwrap();
    compile_test_lib(&test_libs_dir, "test_simple", "42");
    compile_test_lib(&test_libs_dir, "test_negative", "-123");
    compile_test_lib(&test_libs_dir, "test_zero", "0");
    compile_test_lib(&test_libs_dir, "test_max", "32767");
    compile_test_lib(&test_libs_dir, "test_min", "-32768");
    compile_test_lib_vertex(&test_libs_dir);
    compile_test_lib_counter(&test_libs_dir);
    compile_test_lib_no_f(&test_libs_dir);
    println!("cargo:rustc-env=TEST_LIBS_DIR={}", test_libs_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn compile_test_lib(dir: &PathBuf, name: &str, return_value: &str) {
    let source = format!(
        r#"
#[unsafe(no_mangle)]
pub extern "C" fn f(_universe: *mut u8, _vertex: u32) -> i16 {{
    {return_value}
}}
"#
    );
    compile_lib(dir, name, &source);
}

fn compile_test_lib_vertex(dir: &PathBuf) {
    let source = r#"
#[unsafe(no_mangle)]
pub extern "C" fn f(_universe: *mut u8, vertex: u32) -> i16 {
    (vertex % 100) as i16
}
"#;
    compile_lib(dir, "test_vertex", source);
}

fn compile_test_lib_counter(dir: &PathBuf) {
    let source = r#"
static mut COUNTER: i16 = 0;
#[unsafe(no_mangle)]
pub extern "C" fn f(_universe: *mut u8, _vertex: u32) -> i16 {
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}
"#;
    compile_lib(dir, "test_counter", source);
}

fn compile_test_lib_no_f(dir: &PathBuf) {
    let source = r#"
#[unsafe(no_mangle)]
pub extern "C" fn wrong_name(_universe: *mut u8, _vertex: u32) -> i16 {
    42
}
"#;
    compile_lib(dir, "test_no_f", source);
}

fn compile_lib(dir: &PathBuf, name: &str, source: &str) {
    let lib_dir = dir.join(name);
    fs::create_dir_all(&lib_dir).unwrap();
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]
"#,
        name
    );
    let mut cargo_file = fs::File::create(lib_dir.join("Cargo.toml")).unwrap();
    cargo_file.write_all(cargo_toml.as_bytes()).unwrap();
    let src_dir = lib_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let mut lib_file = fs::File::create(src_dir.join("lib.rs")).unwrap();
    lib_file.write_all(source.as_bytes()).unwrap();
    let output = Command::new("cargo")
        .args(["build", "--release", "--manifest-path"])
        .arg(lib_dir.join("Cargo.toml"))
        .output()
        .expect("Failed to compile test library");
    if !output.status.success() {
        panic!(
            "Failed to compile {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
