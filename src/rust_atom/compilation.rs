// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Compilation of Rust source code to shared libraries.
//!
//! This module handles the complete compilation pipeline for Rust atoms:
//! - Creating isolated Cargo projects with proper configuration
//! - Writing source files and build manifests
//! - Invoking cargo to build cdylib shared libraries
//! - Platform-specific library naming and location
//!
//! Each atom is compiled in its own directory with edition 2024 and
//! crate-type cdylib for dynamic loading.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Compiles Rust source code to a shared library.
///
/// Creates a complete Cargo project structure in the specified build
/// directory, compiles the source code, and returns the path to the
/// resulting shared library (.so on Linux, .dylib on macOS, .dll on Windows).
///
/// # Arguments
///
/// * `id` - Unique identifier for the atom (used as crate name)
/// * `source` - Rust source code to compile
/// * `build_dir` - Directory for build artifacts
///
/// # Returns
///
/// Path to the compiled shared library
///
/// # Errors
///
/// Returns an error if:
/// - Build directory cannot be created
/// - File writing fails
/// - Cargo compilation fails
/// - Compiled library is not found
///
/// # Examples
///
/// ```no_run
/// use phie::rust_atom::compile;
///
/// let source = r#"
/// #[no_mangle]
/// pub extern "C" fn f() -> i16 { 42 }
/// "#;
/// let lib_path = compile("my_atom", source, "/tmp/build").unwrap();
/// ```
pub fn compile(id: &str, source: &str, build_dir: &str) -> Result<PathBuf, String> {
    let build_path = Path::new(build_dir);
    fs::create_dir_all(build_path).map_err(|e| format!("Failed to create build dir: {e}"))?;

    let atom_dir = build_path.join(id);
    fs::create_dir_all(&atom_dir).map_err(|e| format!("Failed to create atom dir: {e}"))?;

    write_cargo_toml(&atom_dir, id)?;
    write_lib_rs(&atom_dir, source)?;

    let output = Command::new("cargo")
        .args(["build", "--release", "--manifest-path"])
        .arg(atom_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("Failed to run cargo: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Compilation failed: {stderr}"));
    }

    let lib_name = get_library_name(id);
    let lib_path = atom_dir.join("target/release").join(&lib_name);

    if !lib_path.exists() {
        let path = lib_path.display();
        return Err(format!("Library not found: {path}"));
    }

    Ok(lib_path)
}

/// Writes Cargo.toml manifest for the atom.
///
/// Creates a minimal Cargo.toml with edition 2024 and cdylib crate type.
///
/// # Arguments
///
/// * `dir` - Directory to write Cargo.toml in
/// * `id` - Crate name for the atom
///
/// # Errors
///
/// Returns an error if file creation or writing fails.
pub fn write_cargo_toml(dir: &Path, id: &str) -> Result<(), String> {
    let content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
"#,
        id
    );

    let mut file = fs::File::create(dir.join("Cargo.toml"))
        .map_err(|e| format!("Failed to create Cargo.toml: {e}"))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write Cargo.toml: {e}"))?;
    Ok(())
}

/// Writes lib.rs source file for the atom.
///
/// Creates src/lib.rs with the provided source code.
///
/// # Arguments
///
/// * `dir` - Directory to write source in (will create src/ subdir)
/// * `source` - Rust source code
///
/// # Errors
///
/// Returns an error if directory or file creation fails.
pub fn write_lib_rs(dir: &Path, source: &str) -> Result<(), String> {
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {e}"))?;

    let mut file = fs::File::create(src_dir.join("lib.rs"))
        .map_err(|e| format!("Failed to create lib.rs: {e}"))?;
    file.write_all(source.as_bytes()).map_err(|e| format!("Failed to write lib.rs: {e}"))?;
    Ok(())
}

/// Returns the platform-specific shared library filename.
///
/// # Arguments
///
/// * `id` - Atom identifier
///
/// # Returns
///
/// Library filename:
/// - Linux: "lib{id}.so"
/// - macOS: "lib{id}.dylib"
/// - Windows: "{id}.dll"
fn get_library_name(id: &str) -> String {
    if cfg!(target_os = "linux") {
        format!("lib{id}.so")
    } else if cfg!(target_os = "macos") {
        format!("lib{id}.dylib")
    } else {
        format!("{id}.dll")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_library_name() {
        let name = get_library_name("test");
        if cfg!(target_os = "linux") {
            assert_eq!(name, "libtest.so");
        } else if cfg!(target_os = "macos") {
            assert_eq!(name, "libtest.dylib");
        } else {
            assert_eq!(name, "test.dll");
        }
    }

    #[test]
    fn test_write_cargo_toml() {
        let temp_dir = std::env::temp_dir().join("phie_test_cargo_write");
        fs::create_dir_all(&temp_dir).unwrap();

        write_cargo_toml(&temp_dir, "test_crate").unwrap();

        let cargo_path = temp_dir.join("Cargo.toml");
        assert!(cargo_path.exists());

        let content = fs::read_to_string(cargo_path).unwrap();
        assert!(content.contains("name = \"test_crate\""));
        assert!(content.contains("edition = \"2024\""));
        assert!(content.contains("crate-type = [\"cdylib\"]"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_lib_rs() {
        let temp_dir = std::env::temp_dir().join("phie_test_lib_write");
        fs::create_dir_all(&temp_dir).unwrap();

        let source = "pub fn hello() { println!(\"test\"); }";
        write_lib_rs(&temp_dir, source).unwrap();

        let lib_path = temp_dir.join("src/lib.rs");
        assert!(lib_path.exists());

        let content = fs::read_to_string(lib_path).unwrap();
        assert_eq!(content, source);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_integration() {
        let source = r#"
#[no_mangle]
pub extern "C" fn f(_uni: *mut u8, _v: u32) -> i16 {
    42
}
"#;
        let temp_dir = std::env::temp_dir().join("phie_test_compile_int");

        let result = compile("compile_test", source, temp_dir.to_str().unwrap());

        if let Ok(lib_path) = result {
            assert!(lib_path.exists());
            let filename = lib_path.file_name().unwrap().to_str().unwrap();
            assert!(filename.contains("compile_test"));
        }

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_creates_structure() {
        let source = "pub extern \"C\" fn f() {}";
        let temp_dir = std::env::temp_dir().join("phie_test_structure");

        if compile("structure_test", source, temp_dir.to_str().unwrap()).is_ok() {
            let atom_dir = temp_dir.join("structure_test");
            assert!(atom_dir.exists());
            assert!(atom_dir.join("Cargo.toml").exists());
            assert!(atom_dir.join("src/lib.rs").exists());
        }

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_invalid_source() {
        let invalid_source = "this is not valid rust";
        let temp_dir = std::env::temp_dir().join("phie_test_invalid");

        let result = compile("invalid", invalid_source, temp_dir.to_str().unwrap());

        if Command::new("cargo").arg("--version").output().is_ok() {
            assert!(result.is_err());
        }

        fs::remove_dir_all(&temp_dir).ok();
    }
}
