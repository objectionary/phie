// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Dynamic loading and execution of compiled Rust atoms.
//!
//! This module loads a shared library produced by
//! [`super::compile`] at runtime via `libloading` and calls its entry
//! point through the [`UniverseVTable`] C ABI, so the atom can reach the
//! graph without linking against `phie`.

use crate::data::Data;
use crate::universe::{Universe, UniverseVTable};
use std::path::Path;

/// Function signature for compiled Rust atom entry points.
///
/// Compiled atoms expose a function named `f` with C ABI that accepts:
/// - A pointer to the [`UniverseVTable`] giving access to the graph
/// - A vertex ID (u32) indicating the current execution context
/// - A mutable pointer where the resulting data (i16) is written
///
/// Returns 0 on success and any other value on failure.
pub type RustAtomFn =
    unsafe extern "C" fn(abi: *const UniverseVTable, vertex: u32, out: *mut i16) -> i32;

/// Loads and executes a compiled Rust atom.
///
/// Dynamically loads the shared library at the given path, finds the
/// entry point function named "f", and executes it with the callbacks of
/// the provided universe and the given vertex.
///
/// # Arguments
///
/// * `lib_path` - Path to the compiled shared library
/// * `universe` - Universe the atom will be able to reach
/// * `vertex` - Vertex ID for the execution context
///
/// # Returns
///
/// The data value returned by the atom function
///
/// # Errors
///
/// Returns an error if:
/// - The library file doesn't exist or cannot be loaded
/// - The "f" function symbol is not found
/// - The atom reports a failure
///
/// # Safety
///
/// This function performs unsafe operations:
/// - Dynamic library loading
/// - Symbol resolution
/// - FFI function call with raw pointers
///
/// The caller must ensure the library is compatible and trustworthy.
///
/// # Examples
///
/// ```no_run
/// use phie::rust_atom::execute;
/// use phie::universe::Universe;
/// use std::path::Path;
///
/// let mut universe = Universe::new();
/// let result = execute(Path::new("/tmp/atom.so"), &mut universe, 0);
/// ```
pub fn execute(lib_path: &Path, universe: &mut Universe, vertex: u32) -> Result<Data, String> {
    if !lib_path.exists() {
        let path = lib_path.display();
        return Err(format!("Library not found: {path}"));
    }
    let vtable = UniverseVTable::for_universe(universe);
    unsafe {
        let lib = libloading::Library::new(lib_path)
            .map_err(|e| format!("Failed to load library: {e}"))?;
        let func: libloading::Symbol<RustAtomFn> = lib
            .get(b"f")
            .map_err(|e| format!("Failed to find function f: {e}"))?;
        let mut out: Data = 0;
        let status = func(&vtable, vertex, &mut out);
        if status != 0 {
            let path = lib_path.display();
            return Err(format!("Atom at {path} failed with status {status}"));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust_atom::{compile, write_cargo_toml};
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("phie_{name}"))
    }

    fn compile_fixture(id: &str, source: &str) -> PathBuf {
        compile(id, source, temp_dir("execution").to_str().unwrap())
            .expect("the fixture must compile")
    }

    #[test]
    fn test_execute_nonexistent_library() {
        let mut universe = Universe::new();
        let result = execute(Path::new("/nonexistent/lib.so"), &mut universe, 0);
        let err = result.expect_err("a missing library must be an error");
        assert!(err.contains("not found"), "{err}");
    }

    #[test]
    fn test_execute_returns_atom_value() {
        let lib = compile_fixture(
            "exec_returns",
            "pub fn atom(_uni: &Universe, _v: u32) -> Result<i16, String> { Ok(100) }",
        );
        let mut universe = Universe::new();
        assert_eq!(100, execute(&lib, &mut universe, 0).unwrap());
    }

    #[test]
    fn test_execute_passes_vertex() {
        let lib = compile_fixture(
            "exec_vertex",
            "pub fn atom(_uni: &Universe, v: u32) -> Result<i16, String> { Ok((v % 100) as i16) }",
        );
        let mut universe = Universe::new();
        assert_eq!(7, execute(&lib, &mut universe, 107).unwrap());
    }

    #[test]
    fn test_execute_reports_atom_failure() {
        let lib = compile_fixture(
            "exec_fails",
            "pub fn atom(_uni: &Universe, _v: u32) -> Result<i16, String> { Err(\"no\".into()) }",
        );
        let mut universe = Universe::new();
        let err = execute(&lib, &mut universe, 0).expect_err("a failing atom must error");
        assert!(err.contains("failed with status"), "{err}");
    }

    #[test]
    fn test_execute_atom_reads_through_universe() {
        let lib = compile_fixture(
            "exec_reads",
            r#"pub fn atom(uni: &Universe, v: u32) -> Result<i16, String> {
    let a = uni.dataize(&format!("v{v}.\u{1d6fc}0"))?;
    let b = uni.dataize(&format!("v{v}.\u{1d6fc}1"))?;
    Ok(a + b)
}"#,
        );
        let mut universe = Universe::new();
        let v = universe.next_id();
        let left = universe.next_id();
        let right = universe.next_id();
        universe.add(v).unwrap();
        universe.add(left).unwrap();
        universe.add(right).unwrap();
        universe.put(left, 4).unwrap();
        universe.put(right, 8).unwrap();
        universe.bind(v, left, "𝛼0").unwrap();
        universe.bind(v, right, "𝛼1").unwrap();
        assert_eq!(12, execute(&lib, &mut universe, v).unwrap());
    }

    #[test]
    fn test_execute_atom_writes_through_universe() {
        let lib = compile_fixture(
            "exec_writes",
            r#"pub fn atom(uni: &Universe, v: u32) -> Result<i16, String> {
    let stored = uni.next_id();
    uni.add(stored)?;
    uni.put(stored, 42)?;
    uni.bind(v, stored, "\u{1d6fc}9")?;
    uni.dataize(&format!("v{v}.\u{1d6fc}9"))
}"#,
        );
        let mut universe = Universe::new();
        let v = universe.next_id();
        universe.add(v).unwrap();
        assert_eq!(42, execute(&lib, &mut universe, v).unwrap());
    }

    #[test]
    fn test_execute_fails_when_symbol_missing() {
        let dir = temp_dir("exec_no_symbol");
        let crate_dir = dir.join("exec_no_f");
        fs::create_dir_all(crate_dir.join("src")).unwrap();
        write_cargo_toml(&crate_dir, "exec_no_f").unwrap();
        fs::write(crate_dir.join("src/lib.rs"), "pub fn not_the_entry() {}").unwrap();
        let output = std::process::Command::new("cargo")
            .args(["build", "--release", "--manifest-path"])
            .arg(crate_dir.join("Cargo.toml"))
            .output()
            .unwrap();
        assert!(output.status.success());
        let lib = crate_dir.join("target/release");
        let name = if cfg!(target_os = "windows") {
            "exec_no_f.dll"
        } else if cfg!(target_os = "macos") {
            "libexec_no_f.dylib"
        } else {
            "libexec_no_f.so"
        };
        let mut universe = Universe::new();
        let err = execute(&lib.join(name), &mut universe, 0)
            .expect_err("a library without the entry symbol must fail");
        assert!(err.contains("Failed to find function f"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }
}
