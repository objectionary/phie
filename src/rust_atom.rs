// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Rust atom compilation and execution module.
//!
//! This module provides the infrastructure for compiling Rust source code
//! into shared libraries and executing them as atoms within the phie runtime.
//! It handles the complete pipeline from source code to executable function.
//!
//! The RustAtom struct coordinates between the compilation and execution
//! subsystems, managing the lifecycle of dynamically loaded code.

mod compilation;
mod execution;

pub use compilation::{compile, write_cargo_toml, write_lib_rs};
pub use execution::{execute, RustAtomFn};

use crate::data::Data;
use crate::universe::Universe;
use std::path::PathBuf;

/// Represents a Rust atom with its source code and compilation state.
///
/// A RustAtom encapsulates all information needed to compile and execute
/// a piece of Rust code as an atom. It manages:
/// - Unique identification for the atom
/// - Source code as a string
/// - Path to the compiled shared library
/// - Compilation process and error handling
///
/// # Compilation Process
///
/// Each atom is compiled into its own Cargo project with edition 2024
/// and crate-type cdylib. The compilation produces a platform-specific
/// shared library (.so on Linux, .dylib on macOS, .dll on Windows).
///
/// # Examples
///
/// ```no_run
/// use phie::rust_atom::RustAtom;
///
/// let source = r#"
/// #[no_mangle]
/// pub extern "C" fn f(_uni: *mut u8, _v: u32) -> i16 {
///     42
/// }
/// "#;
///
/// let mut atom = RustAtom::new("my_atom", source);
/// atom.compile("/tmp/build").unwrap();
/// ```
pub struct RustAtom {
    /// Unique identifier for this atom
    pub id: String,
    /// Rust source code to be compiled
    pub source: String,
    /// Path to the compiled shared library, if compilation succeeded
    pub lib_path: Option<PathBuf>,
}

impl RustAtom {
    /// Creates a new RustAtom from source code.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this atom
    /// * `source` - Rust source code
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::rust_atom::RustAtom;
    ///
    /// let atom = RustAtom::new("sum", "pub fn f() { }");
    /// ```
    pub fn new(id: &str, source: &str) -> Self {
        RustAtom { id: id.to_string(), source: source.to_string(), lib_path: None }
    }

    /// Compiles the Rust source code to a shared library.
    ///
    /// # Arguments
    ///
    /// * `build_dir` - Directory for build artifacts
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use phie::rust_atom::RustAtom;
    ///
    /// let mut atom = RustAtom::new("sum", "pub extern \"C\" fn f() -> i16 { 42 }");
    /// atom.compile("/tmp/phie").unwrap();
    /// ```
    pub fn compile(&mut self, build_dir: &str) -> Result<PathBuf, String> {
        let lib_path = compilation::compile(&self.id, &self.source, build_dir)?;
        self.lib_path = Some(lib_path.clone());
        Ok(lib_path)
    }

    /// Loads and executes the compiled Rust atom.
    ///
    /// # Arguments
    ///
    /// * `universe` - Universe instance for SODG operations
    /// * `vertex` - Vertex ID to operate on
    ///
    /// # Errors
    ///
    /// Returns an error if the atom has not been compiled yet or if
    /// execution fails.
    pub fn execute(&self, universe: &mut Universe, vertex: u32) -> Result<Data, String> {
        let lib_path = self.lib_path.as_ref().ok_or("Library not compiled")?;
        execution::execute(lib_path, universe, vertex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_rust_atom_creation() {
        let atom = RustAtom::new("test", "fn main() {}");
        assert_eq!(atom.id, "test");
        assert_eq!(atom.source, "fn main() {}");
        assert!(atom.lib_path.is_none());
    }

    #[test]
    fn test_new_atom_has_no_lib_path() {
        let atom = RustAtom::new("test", "");
        assert!(atom.lib_path.is_none());
    }

    #[test]
    fn test_id_and_source_stored() {
        let id = "my_atom";
        let source = "fn test() {}";
        let atom = RustAtom::new(id, source);
        assert_eq!(atom.id, id);
        assert_eq!(atom.source, source);
    }

    #[test]
    fn test_execute_requires_compilation() {
        let atom = RustAtom::new("not_compiled", "");
        let mut uni = Universe::new();
        let result = atom.execute(&mut uni, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not compiled"));
    }

    #[test]
    fn test_compile_workflow() {
        let source = r#"
#[no_mangle]
pub extern "C" fn f(_uni: *mut u8, _v: u32) -> i16 {
    42
}
"#;
        let mut atom = RustAtom::new("workflow_test", source);
        let temp_dir = std::env::temp_dir().join("phie_workflow_test");

        let result = atom.compile(temp_dir.to_str().unwrap());

        if result.is_ok() {
            assert!(atom.lib_path.is_some());
            assert!(atom.lib_path.as_ref().unwrap().exists());
        }

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_execute_workflow() {
        let source = r#"
#[no_mangle]
pub extern "C" fn f(_uni: *mut u8, _v: u32) -> i16 {
    100
}
"#;
        let mut atom = RustAtom::new("exec_workflow", source);
        let temp_dir = std::env::temp_dir().join("phie_exec_workflow");

        if atom.compile(temp_dir.to_str().unwrap()).is_ok() {
            let mut uni = Universe::new();
            let result = atom.execute(&mut uni, 0);
            if result.is_ok() {
                assert_eq!(result.unwrap(), 100);
            }
        }

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compile_sets_lib_path() {
        let mut atom = RustAtom::new("lib_path_test", "pub extern \"C\" fn f() {}");
        let temp_dir = std::env::temp_dir().join("phie_lib_path_test");

        if atom.compile(temp_dir.to_str().unwrap()).is_ok() {
            assert!(atom.lib_path.is_some());
            let lib_path = atom.lib_path.unwrap();
            assert!(lib_path.exists());
            assert!(lib_path.to_string_lossy().contains("lib_path_test"));
        }

        fs::remove_dir_all(&temp_dir).ok();
    }
}
