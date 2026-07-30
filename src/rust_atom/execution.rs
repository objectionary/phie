// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Dynamic loading and execution of compiled Rust atoms.
//!
//! This module handles loading shared libraries at runtime via libloading
//! and provides safe FFI wrappers for executing atom functions. The execution
//! interface bridges between phie's Universe and the compiled Rust code.

use crate::data::Data;
use crate::universe::Universe;
use std::path::Path;

/// Function signature for compiled Rust atom entry points.
///
/// Compiled atoms must expose a function with C ABI that accepts:
/// - A mutable pointer to Universe for graph operations
/// - A vertex ID (u32) indicating the current execution context
///
/// Returns i16 (Data) representing the computed result.
///
/// # Safety
///
/// This is an unsafe extern C function pointer. Callers must ensure:
/// - The Universe pointer is valid and properly aligned
/// - The Universe is not accessed from multiple threads simultaneously
/// - The vertex ID is valid within the current graph
/// - The lifetime of the Universe exceeds the function call
pub type RustAtomFn = unsafe extern "C" fn(*mut Universe, u32) -> i16;

/// Loads and executes a compiled Rust atom.
///
/// Dynamically loads the shared library at the given path, finds the
/// entry point function named "f", and executes it with the provided
/// Universe and vertex ID.
///
/// # Arguments
///
/// * `lib_path` - Path to the compiled shared library
/// * `universe` - Mutable reference to Universe for graph operations
/// * `vertex` - Vertex ID for the execution context
///
/// # Returns
///
/// The i16 result value returned by the atom function
///
/// # Errors
///
/// Returns an error if:
/// - The library file doesn't exist or cannot be loaded
/// - The "f" function symbol is not found
/// - The function signature is incorrect
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
    unsafe {
        let lib = libloading::Library::new(lib_path)
            .map_err(|e| format!("Failed to load library: {e}"))?;
        let func: libloading::Symbol<RustAtomFn> = lib
            .get(b"f")
            .map_err(|e| format!("Failed to find function f: {e}"))?;
        let result = func(universe as *mut Universe, vertex);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_lib_path(name: &str) -> PathBuf {
        let test_libs_dir = env!("TEST_LIBS_DIR");
        let lib_name = if cfg!(target_os = "linux") {
            format!("lib{name}.so")
        } else if cfg!(target_os = "macos") {
            format!("lib{name}.dylib")
        } else {
            format!("{name}.dll")
        };
        PathBuf::from(test_libs_dir)
            .join(name)
            .join("target/release")
            .join(lib_name)
    }

    #[test]
    fn test_execute_nonexistent_library() {
        let mut universe = Universe::new();
        let result = execute(Path::new("/nonexistent/lib.so"), &mut universe, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_execute_requires_valid_path() {
        let mut universe = Universe::new();
        let invalid_path = PathBuf::from("/tmp/does_not_exist_phie_test.so");
        let result = execute(&invalid_path, &mut universe, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_successful_simple_function() {
        let lib_path = get_test_lib_path("test_simple");
        let mut universe = Universe::new();
        let result = execute(&lib_path, &mut universe, 0).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_execute_with_different_return_values() {
        let lib_path = get_test_lib_path("test_negative");
        let mut universe = Universe::new();
        let result = execute(&lib_path, &mut universe, 0).unwrap();
        assert_eq!(result, -123);
    }

    #[test]
    fn test_execute_with_zero_return() {
        let lib_path = get_test_lib_path("test_zero");
        let mut universe = Universe::new();
        let result = execute(&lib_path, &mut universe, 0).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_execute_with_max_i16_value() {
        let lib_path = get_test_lib_path("test_max");
        let mut universe = Universe::new();
        let result = execute(&lib_path, &mut universe, 0).unwrap();
        assert_eq!(result, i16::MAX);
    }

    #[test]
    fn test_execute_with_min_i16_value() {
        let lib_path = get_test_lib_path("test_min");
        let mut universe = Universe::new();
        let result = execute(&lib_path, &mut universe, 0).unwrap();
        assert_eq!(result, i16::MIN);
    }

    #[test]
    fn test_execute_with_different_vertex_ids() {
        let lib_path = get_test_lib_path("test_vertex");
        let mut universe = Universe::new();
        let result_0 = execute(&lib_path, &mut universe, 0).unwrap();
        assert_eq!(result_0, 0);
        let result_42 = execute(&lib_path, &mut universe, 42).unwrap();
        assert_eq!(result_42, 42);
        let result_max = execute(&lib_path, &mut universe, u32::MAX).unwrap();
        assert_eq!(result_max, (u32::MAX % 100) as i16);
    }

    #[test]
    fn test_execute_invalid_library_file() {
        let temp_dir = std::env::temp_dir().join("phie_test_exec_invalid");
        fs::create_dir_all(&temp_dir).unwrap();
        let invalid_lib = temp_dir.join("invalid.so");
        fs::write(&invalid_lib, b"not a valid library").unwrap();
        let mut universe = Universe::new();
        let result = execute(&invalid_lib, &mut universe, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to load library"));
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_execute_library_without_f_function() {
        let lib_path = get_test_lib_path("test_no_f");
        let mut universe = Universe::new();
        let result = execute(&lib_path, &mut universe, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to find function f"));
    }

    #[test]
    fn test_execute_directory_instead_of_file() {
        let temp_dir = std::env::temp_dir().join("phie_test_exec_dir");
        fs::create_dir_all(&temp_dir).unwrap();
        let mut universe = Universe::new();
        let result = execute(&temp_dir, &mut universe, 0);
        assert!(result.is_err());
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_execute_empty_path() {
        let mut universe = Universe::new();
        let empty_path = PathBuf::from("");
        let result = execute(&empty_path, &mut universe, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_multiple_calls_same_library() {
        let lib_path = get_test_lib_path("test_counter");
        let mut universe = Universe::new();
        let result1 = execute(&lib_path, &mut universe, 0).unwrap();
        let result2 = execute(&lib_path, &mut universe, 0).unwrap();
        assert!(result1 > 0);
        assert!(result2 > 0);
    }

    #[test]
    fn test_execute_preserves_error_message_on_nonexistent() {
        let mut universe = Universe::new();
        let path = Path::new("/definitely/does/not/exist/library.so");
        let result = execute(path, &mut universe, 0);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Library not found"));
        assert!(error.contains("/definitely/does/not/exist/library.so"));
    }
}
