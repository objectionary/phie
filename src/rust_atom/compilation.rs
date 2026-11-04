// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Compilation of Rust source code to shared libraries.
//!
//! This module handles the complete compilation pipeline for Rust atoms:
//! - Creating isolated Cargo projects with proper configuration
//! - Wrapping the atom source with the [`UniverseVTable`] preamble and the
//!   `extern "C"` entry point the loader looks for
//! - Invoking cargo to build cdylib shared libraries
//! - Platform-specific library naming and location
//!
//! Each atom is compiled in its own directory with edition 2024 and
//! crate-type cdylib for dynamic loading. The atom source itself must
//! define `pub fn atom(uni: &Universe, v: u32) -> Result<i16, String>`;
//! everything else — the ABI struct, its safe wrapper, and the exported
//! symbol — is generated around it.
//!
//! [`UniverseVTable`]: crate::universe::UniverseVTable

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The code written before the atom source into every generated crate.
///
/// It mirrors `crate::universe::abi::UniverseVTable` field by field, which
/// is what allows the atom to call graph operations without depending on
/// `phie`. Keep the two definitions in sync.
const PREAMBLE: &str = r#"use std::os::raw::{c_char, c_void};
use std::ffi::CString;
#[repr(C)]
pub struct UniverseVTable {
    pub ctx: *mut c_void,
    pub dataize: unsafe extern "C" fn(ctx: *mut c_void, path: *const c_char, out: *mut i16) -> i32,
    pub add: unsafe extern "C" fn(ctx: *mut c_void, v: u32) -> i32,
    pub put: unsafe extern "C" fn(ctx: *mut c_void, v: u32, data: i16) -> i32,
    pub bind: unsafe extern "C" fn(ctx: *mut c_void, from: u32, to: u32, label: *const c_char) -> i32,
    pub next_id: unsafe extern "C" fn(ctx: *mut c_void) -> u32,
}
pub struct Universe<'a> {
    vtable: &'a UniverseVTable,
}
impl<'a> Universe<'a> {
    fn from_vtable(vtable: *const UniverseVTable) -> Self {
        Self { vtable: unsafe { &*vtable } }
    }
    pub fn dataize(&self, path: &str) -> Result<i16, String> {
        let name = CString::new(path).map_err(|e| e.to_string())?;
        let mut out: i16 = 0;
        let status = unsafe { (self.vtable.dataize)(self.vtable.ctx, name.as_ptr(), &mut out) };
        if status == 0 { Ok(out) } else { Err(format!("Can't dataize '{path}'")) }
    }
    pub fn add(&self, v: u32) -> Result<(), String> {
        let status = unsafe { (self.vtable.add)(self.vtable.ctx, v) };
        if status == 0 { Ok(()) } else { Err(format!("Can't add vertex {v}")) }
    }
    pub fn put(&self, v: u32, data: i16) -> Result<(), String> {
        let status = unsafe { (self.vtable.put)(self.vtable.ctx, v, data) };
        if status == 0 { Ok(()) } else { Err(format!("Can't put data into vertex {v}")) }
    }
    pub fn bind(&self, from: u32, to: u32, label: &str) -> Result<(), String> {
        let name = CString::new(label).map_err(|e| e.to_string())?;
        let status = unsafe { (self.vtable.bind)(self.vtable.ctx, from, to, name.as_ptr()) };
        if status == 0 { Ok(()) } else { Err(format!("Can't bind {from} to {to} by '{label}'")) }
    }
    pub fn next_id(&self) -> u32 {
        unsafe { (self.vtable.next_id)(self.vtable.ctx) }
    }
}
"#;

/// The code written after the atom source into every generated crate.
///
/// It exports the symbol the loader resolves and turns the atom's
/// `Result` into a status code plus an out-parameter.
const ENTRY_POINT: &str = r#"#[unsafe(no_mangle)]
pub extern "C" fn f(abi: *const UniverseVTable, v: u32, out: *mut i16) -> i32 {
    if abi.is_null() || out.is_null() {
        return 1;
    }
    let uni = Universe::from_vtable(abi);
    match atom(&uni, v) {
        Ok(data) => {
            unsafe { *out = data };
            0
        }
        Err(_) => 1,
    }
}
"#;

/// Compiles Rust source code to a shared library.
///
/// Creates a complete Cargo project structure in the specified build
/// directory, wraps the source with the ABI preamble and the entry point,
/// compiles the source code, and returns the path to the resulting shared
/// library (.so on Linux, .dylib on macOS, .dll on Windows).
///
/// # Arguments
///
/// * `id` - Unique identifier for the atom (used as crate name)
/// * `source` - Rust source code defining `pub fn atom(&Universe, u32)`
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
/// let source = "pub fn atom(_uni: &Universe, _v: u32) -> Result<i16, String> { Ok(42) }";
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
/// The empty `[workspace]` table detaches the atom from any surrounding
/// workspace, so it builds the same way wherever the build directory is.
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
name = "{id}"
version = "0.0.0"
edition = "2024"

[workspace]

[lib]
crate-type = ["cdylib"]
"#
    );
    let mut file = fs::File::create(dir.join("Cargo.toml"))
        .map_err(|e| format!("Failed to create Cargo.toml: {e}"))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write Cargo.toml: {e}"))?;
    Ok(())
}

/// Writes lib.rs source file for the atom.
///
/// Creates src/lib.rs with the ABI preamble, the provided source code,
/// and the generated entry point around them.
///
/// # Arguments
///
/// * `dir` - Directory to write source in (will create src/ subdir)
/// * `source` - Rust source code defining `pub fn atom`
///
/// # Errors
///
/// Returns an error if directory or file creation fails.
pub fn write_lib_rs(dir: &Path, source: &str) -> Result<(), String> {
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {e}"))?;
    let mut file = fs::File::create(src_dir.join("lib.rs"))
        .map_err(|e| format!("Failed to create lib.rs: {e}"))?;
    file.write_all(PREAMBLE.as_bytes())
        .map_err(|e| format!("Failed to write preamble: {e}"))?;
    file.write_all(source.as_bytes())
        .map_err(|e| format!("Failed to write source: {e}"))?;
    file.write_all(ENTRY_POINT.as_bytes())
        .map_err(|e| format!("Failed to write entry point: {e}"))?;
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

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("phie_{name}"))
    }

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
        let dir = temp_dir("test_cargo_write");
        fs::create_dir_all(&dir).unwrap();
        write_cargo_toml(&dir, "test_crate").unwrap();
        let content = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(content.contains("name = \"test_crate\""));
        assert!(content.contains("edition = \"2024\""));
        assert!(content.contains("crate-type = [\"cdylib\"]"));
        assert!(content.contains("[workspace]"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_write_lib_rs_wraps_source() {
        let dir = temp_dir("test_lib_write");
        fs::create_dir_all(&dir).unwrap();
        let source = "pub fn atom(_uni: &Universe, _v: u32) -> Result<i16, String> { Ok(0) }";
        write_lib_rs(&dir, source).unwrap();
        let content = fs::read_to_string(dir.join("src/lib.rs")).unwrap();
        assert!(content.contains("#[repr(C)]"));
        assert!(content.contains(source));
        assert!(content.contains("#[unsafe(no_mangle)]"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_compile_produces_library() {
        let source = "pub fn atom(_uni: &Universe, _v: u32) -> Result<i16, String> { Ok(42) }";
        let mut atom = super::super::RustAtom::new("compile_produces", source);
        let dir = temp_dir("test_compile_produces");
        let lib_path = atom
            .compile(dir.to_str().unwrap())
            .expect("the atom must compile");
        assert!(lib_path.exists());
        assert!(lib_path.to_string_lossy().contains("compile_produces"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_compile_fails_on_broken_source() {
        let source = "this is not rust";
        let mut atom = super::super::RustAtom::new("compile_broken", source);
        let dir = temp_dir("test_compile_broken");
        let err = atom
            .compile(dir.to_str().unwrap())
            .expect_err("broken source must not compile");
        assert!(err.contains("Compilation failed"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }
}
