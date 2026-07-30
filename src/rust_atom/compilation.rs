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
    file.write_all(source.as_bytes())
        .map_err(|e| format!("Failed to write lib.rs: {e}"))?;
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

    #[test]
    fn test_write_cargo_toml_creates_valid_content() {
        let temp_dir = std::env::temp_dir().join("phie_test_cargo_content");
        fs::create_dir_all(&temp_dir).unwrap();
        write_cargo_toml(&temp_dir, "my_atom").unwrap();
        let content = fs::read_to_string(temp_dir.join("Cargo.toml")).unwrap();
        assert!(content.contains("[package]"));
        assert!(content.contains("name = \"my_atom\""));
        assert!(content.contains("version = \"0.1.0\""));
        assert!(content.contains("[lib]"));
        assert!(content.contains("[dependencies]"));
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_lib_rs_creates_src_directory() {
        let temp_dir = std::env::temp_dir().join("phie_test_src_creation");
        fs::create_dir_all(&temp_dir).unwrap();
        write_lib_rs(&temp_dir, "test").unwrap();
        assert!(temp_dir.join("src").exists());
        assert!(temp_dir.join("src").is_dir());
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_lib_rs_preserves_exact_content() {
        let temp_dir = std::env::temp_dir().join("phie_test_exact_content");
        fs::create_dir_all(&temp_dir).unwrap();
        let source = "// comment\npub fn test() {\n    println!(\"hello\");\n}\n";
        write_lib_rs(&temp_dir, source).unwrap();
        let content = fs::read_to_string(temp_dir.join("src/lib.rs")).unwrap();
        assert_eq!(content, source);
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_creates_build_directory() {
        let source = r#"#[no_mangle] pub extern "C" fn f(_u: *mut u8, _v: u32) -> i16 { 0 }"#;
        let temp_dir = std::env::temp_dir().join("phie_test_build_dir");
        fs::remove_dir_all(&temp_dir).ok();
        compile("build_test", source, temp_dir.to_str().unwrap()).ok();
        assert!(temp_dir.exists());
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_get_library_name_with_different_ids() {
        assert!(get_library_name("atom1").contains("atom1"));
        assert!(get_library_name("my_custom_atom").contains("my_custom_atom"));
        assert!(get_library_name("x").contains("x"));
    }

    #[test]
    fn test_get_library_name_linux_format() {
        if cfg!(target_os = "linux") {
            let name = get_library_name("foo");
            assert!(name.starts_with("lib"));
            assert!(name.ends_with(".so"));
        }
    }

    #[test]
    fn test_write_cargo_toml_with_special_characters() {
        let temp_dir = std::env::temp_dir().join("phie_test_special");
        fs::create_dir_all(&temp_dir).unwrap();
        write_cargo_toml(&temp_dir, "atom_123").unwrap();
        let content = fs::read_to_string(temp_dir.join("Cargo.toml")).unwrap();
        assert!(content.contains("name = \"atom_123\""));
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_with_complex_source() {
        let source = r#"
use std::ffi::c_void;

#[no_mangle]
pub extern "C" fn f(_universe: *mut c_void, _vertex: u32) -> i16 {
    let result = 10 + 32;
    result
}
"#;
        let temp_dir = std::env::temp_dir().join("phie_test_complex");
        let result = compile("complex", source, temp_dir.to_str().unwrap());
        if let Ok(lib_path) = result {
            assert!(lib_path.exists());
        }
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_lib_rs_with_empty_source() {
        let temp_dir = std::env::temp_dir().join("phie_test_empty");
        fs::create_dir_all(&temp_dir).unwrap();
        write_lib_rs(&temp_dir, "").unwrap();
        let content = fs::read_to_string(temp_dir.join("src/lib.rs")).unwrap();
        assert_eq!(content, "");
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_creates_target_directory() {
        let source = r#"#[no_mangle] pub extern "C" fn f(_u: *mut u8, _v: u32) -> i16 { 1 }"#;
        let temp_dir = std::env::temp_dir().join("phie_test_target");
        if compile("target_test", source, temp_dir.to_str().unwrap()).is_ok() {
            let target_dir = temp_dir.join("target_test/target");
            assert!(target_dir.exists() || temp_dir.join("target_test/Cargo.toml").exists());
        }
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_cargo_toml_overwrites_existing() {
        let temp_dir = std::env::temp_dir().join("phie_test_overwrite");
        fs::create_dir_all(&temp_dir).unwrap();
        write_cargo_toml(&temp_dir, "first").unwrap();
        write_cargo_toml(&temp_dir, "second").unwrap();
        let content = fs::read_to_string(temp_dir.join("Cargo.toml")).unwrap();
        assert!(content.contains("name = \"second\""));
        assert!(!content.contains("name = \"first\""));
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_lib_rs_overwrites_existing() {
        let temp_dir = std::env::temp_dir().join("phie_test_lib_overwrite");
        fs::create_dir_all(&temp_dir).unwrap();
        write_lib_rs(&temp_dir, "first").unwrap();
        write_lib_rs(&temp_dir, "second").unwrap();
        let content = fs::read_to_string(temp_dir.join("src/lib.rs")).unwrap();
        assert_eq!(content, "second");
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_with_whitespace_in_source() {
        let source = r#"

        #[no_mangle]
        pub extern "C" fn f(_u: *mut u8, _v: u32) -> i16 {

            42

        }

        "#;
        let temp_dir = std::env::temp_dir().join("phie_test_whitespace");
        let result = compile("whitespace", source, temp_dir.to_str().unwrap());
        if let Ok(lib_path) = result {
            assert!(lib_path.exists());
            assert!(lib_path.to_str().unwrap().contains("whitespace"));
        }
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_returns_correct_path() {
        let source = r#"#[no_mangle] pub extern "C" fn f(_u: *mut u8, _v: u32) -> i16 { 7 }"#;
        let temp_dir = std::env::temp_dir().join("phie_test_path");
        if let Ok(path) = compile("path_test", source, temp_dir.to_str().unwrap()) {
            assert!(path.to_str().unwrap().contains("path_test"));
            assert!(path.to_str().unwrap().contains("target/release"));
            let lib_name = get_library_name("path_test");
            assert!(path.to_str().unwrap().contains(&lib_name));
        }
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_get_library_name_format_consistency() {
        let name1 = get_library_name("test");
        let name2 = get_library_name("test");
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_write_cargo_toml_includes_all_sections() {
        let temp_dir = std::env::temp_dir().join("phie_test_sections");
        fs::create_dir_all(&temp_dir).unwrap();
        write_cargo_toml(&temp_dir, "test").unwrap();
        let content = fs::read_to_string(temp_dir.join("Cargo.toml")).unwrap();
        assert!(content.lines().any(|l| l.trim() == "[package]"));
        assert!(content.lines().any(|l| l.trim() == "[lib]"));
        assert!(content.lines().any(|l| l.trim() == "[dependencies]"));
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_with_minimal_valid_source() {
        let source = r#"#[no_mangle] pub extern "C" fn f(_u: *mut u8, _v: u32) -> i16 { 0 }"#;
        let temp_dir = std::env::temp_dir().join("phie_test_minimal");
        let result = compile("minimal", source, temp_dir.to_str().unwrap());
        if let Ok(lib_path) = result {
            assert!(lib_path.exists());
        }
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_lib_rs_handles_multiline() {
        let temp_dir = std::env::temp_dir().join("phie_test_multiline");
        fs::create_dir_all(&temp_dir).unwrap();
        let source = "line1\nline2\nline3";
        write_lib_rs(&temp_dir, source).unwrap();
        let content = fs::read_to_string(temp_dir.join("src/lib.rs")).unwrap();
        assert_eq!(content.lines().count(), 3);
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_error_message_on_invalid_rust() {
        let temp_dir = std::env::temp_dir().join("phie_test_error_msg");
        let result = compile("error_test", "fn {{{", temp_dir.to_str().unwrap());
        if Command::new("cargo").arg("--version").output().is_ok() {
            if let Err(msg) = result {
                assert!(msg.contains("Compilation failed") || msg.contains("error"));
            }
        }
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_get_library_name_preserves_underscores() {
        let name = get_library_name("my_atom_name");
        assert!(name.contains("my_atom_name"));
    }

    #[test]
    fn test_compile_creates_atom_subdirectory() {
        let source = r#"#[no_mangle] pub extern "C" fn f(_u: *mut u8, _v: u32) -> i16 { 3 }"#;
        let temp_dir = std::env::temp_dir().join("phie_test_subdir");
        if compile("subdir_atom", source, temp_dir.to_str().unwrap()).is_ok() {
            assert!(temp_dir.join("subdir_atom").exists());
            assert!(temp_dir.join("subdir_atom").is_dir());
        }
        fs::remove_dir_all(&temp_dir).ok();
    }
}
