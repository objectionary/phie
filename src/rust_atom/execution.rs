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
/// use phie::rust_atom::execution::execute;
/// use phie::universe::Universe;
/// use std::path::Path;
///
/// let mut universe = Universe::new();
/// let result = execute(Path::new("/tmp/atom.so"), &mut universe, 0);
/// ```
pub fn execute(lib_path: &Path, universe: &mut Universe, vertex: u32) -> Result<Data, String> {
    if !lib_path.exists() {
        return Err(format!("Library not found: {}", lib_path.display()));
    }

    unsafe {
        let lib = libloading::Library::new(lib_path)
            .map_err(|e| format!("Failed to load library: {}", e))?;

        let func: libloading::Symbol<RustAtomFn> =
            lib.get(b"f").map_err(|e| format!("Failed to find function f: {}", e))?;

        let result = func(universe as *mut Universe, vertex);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
}
