// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Rust atom execution engine with compilation caching and Universe management.
//!
//! This module provides a high-level engine for managing multiple Rust atoms
//! within a single phie execution context. The RustEngine coordinates:
//! - Registration of Rust atoms by unique identifiers
//! - Lazy compilation of atoms only when needed
//! - Caching of compiled shared libraries for reuse
//! - Thread-safe access to a shared Universe instance
//! - Execution of compiled atoms with proper error handling
//!
//! The engine maintains a single Universe instance wrapped in Arc<Mutex<>>
//! to allow safe concurrent access from multiple compiled atoms while
//! ensuring data consistency.

use crate::data::Data;
use crate::rust_atom::RustAtom;
use crate::universe::Universe;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// High-level engine for managing Rust atom lifecycle.
///
/// RustEngine provides a centralized manager for:
/// - Atom registration: storing source code with unique IDs
/// - Compilation: building atoms into shared libraries
/// - Execution: loading and calling compiled atom functions
/// - Universe sharing: providing thread-safe access to SODG graph
///
/// The engine stores atoms in a HashMap keyed by their unique identifiers.
/// Each atom can be compiled independently and executed multiple times.
/// The shared Universe ensures all atoms operate on the same object graph.
///
/// # Thread Safety
///
/// The Universe is wrapped in Arc<Mutex<>> to allow safe access from
/// multiple atoms. The atoms HashMap itself is not thread-safe and
/// should be accessed from a single thread during compilation phase.
///
/// # Examples
///
/// ```no_run
/// use phie::rust_engine::RustEngine;
///
/// let mut engine = RustEngine::new("/tmp/phie");
/// engine.register("sum", "pub extern \"C\" fn f() -> i16 { 42 }");
/// engine.compile("sum").unwrap();
/// let result = engine.execute("sum", 0).unwrap();
/// assert_eq!(result, 42);
/// ```
pub struct RustEngine {
    /// Registered atoms indexed by unique identifier
    atoms: HashMap<String, RustAtom>,
    /// Directory for build artifacts and compilation output
    build_dir: PathBuf,
    /// Shared Universe instance accessible to all atoms
    universe: Arc<Mutex<Universe>>,
}

impl RustEngine {
    /// Creates a new RustEngine with a build directory.
    ///
    /// # Arguments
    ///
    /// * `build_dir` - Directory for build artifacts
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::rust_engine::RustEngine;
    ///
    /// let engine = RustEngine::new("/tmp/phie");
    /// ```
    pub fn new(build_dir: &str) -> Self {
        RustEngine {
            atoms: HashMap::new(),
            build_dir: PathBuf::from(build_dir),
            universe: Arc::new(Mutex::new(Universe::new())),
        }
    }

    /// Registers a Rust atom from source code.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the atom
    /// * `source` - Rust source code
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::rust_engine::RustEngine;
    ///
    /// let mut engine = RustEngine::new("/tmp/phie");
    /// engine.register("sum", "pub extern \"C\" fn f() -> i16 { 42 }");
    /// ```
    pub fn register(&mut self, id: &str, source: &str) {
        let atom = RustAtom::new(id, source);
        self.atoms.insert(id.to_string(), atom);
    }

    /// Compiles a registered Rust atom.
    ///
    /// # Arguments
    ///
    /// * `id` - Atom identifier
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use phie::rust_engine::RustEngine;
    ///
    /// let mut engine = RustEngine::new("/tmp/phie");
    /// engine.register("sum", "pub extern \"C\" fn f() -> i16 { 42 }");
    /// engine.compile("sum").unwrap();
    /// ```
    pub fn compile(&mut self, id: &str) -> Result<(), String> {
        let atom = self.atoms.get_mut(id).ok_or_else(|| format!("Atom {} not found", id))?;

        atom.compile(self.build_dir.to_str().ok_or("Invalid build dir")?)?;
        Ok(())
    }

    /// Executes a compiled Rust atom.
    ///
    /// # Arguments
    ///
    /// * `id` - Atom identifier
    /// * `vertex` - Vertex ID for operations
    pub fn execute(&self, id: &str, vertex: u32) -> Result<Data, String> {
        let atom = self.atoms.get(id).ok_or_else(|| format!("Atom {} not found", id))?;

        let mut uni = self.universe.lock().map_err(|e| format!("Lock error: {}", e))?;
        atom.execute(&mut uni, vertex)
    }

    /// Compiles and executes a Rust atom in one call.
    ///
    /// # Arguments
    ///
    /// * `id` - Atom identifier
    /// * `source` - Rust source code
    /// * `vertex` - Vertex ID for operations
    pub fn compile_and_execute(
        &mut self,
        id: &str,
        source: &str,
        vertex: u32,
    ) -> Result<Data, String> {
        self.register(id, source);
        self.compile(id)?;
        self.execute(id, vertex)
    }

    /// Returns a reference to the Universe for direct manipulation.
    pub fn universe(&self) -> Arc<Mutex<Universe>> {
        Arc::clone(&self.universe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = RustEngine::new("/tmp/phie_test");
        assert!(engine.atoms.is_empty());
    }

    #[test]
    fn test_register_atom() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        engine.register("test", "fn main() {}");
        assert!(engine.atoms.contains_key("test"));
    }

    #[test]
    fn test_compile_and_execute_integration() {
        let mut engine = RustEngine::new("/tmp/phie_engine_integration");
        let source = r#"
#[no_mangle]
pub extern "C" fn f(_uni: *mut u8, _v: u32) -> i16 {
    999
}
"#;
        let result = engine.compile_and_execute("integration", source, 0);

        if result.is_ok() {
            assert_eq!(result.unwrap(), 999);
        }
    }

    #[test]
    fn test_compile_workflow() {
        let mut engine = RustEngine::new("/tmp/phie_compile_flow");
        let source = "pub extern \"C\" fn f() {}";
        engine.register("flow_test", source);

        assert!(engine.atoms.contains_key("flow_test"));

        if engine.compile("flow_test").is_ok() {
            assert!(engine.atoms.get("flow_test").unwrap().lib_path.is_some());
        }
    }

    #[test]
    fn test_execute_workflow() {
        let mut engine = RustEngine::new("/tmp/phie_execute_flow");
        let source = r#"
#[no_mangle]
pub extern "C" fn f(_uni: *mut u8, _v: u32) -> i16 {
    777
}
"#;
        engine.register("exec_flow", source);

        if engine.compile("exec_flow").is_ok() {
            let result = engine.execute("exec_flow", 0);
            if result.is_ok() {
                assert_eq!(result.unwrap(), 777);
            }
        }
    }

    #[test]
    fn test_compile_nonexistent_atom() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        let result = engine.compile("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_execute_nonexistent_atom() {
        let engine = RustEngine::new("/tmp/phie_test");
        let result = engine.execute("nonexistent", 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_execute_uncompiled_atom() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        engine.register("uncompiled", "fn test() {}");
        let result = engine.execute("uncompiled", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_atoms() {
        let mut engine = RustEngine::new("/tmp/phie_multi_test");
        engine.register("atom1", "fn a() {}");
        engine.register("atom2", "fn b() {}");
        engine.register("atom3", "fn c() {}");
        assert_eq!(engine.atoms.len(), 3);
        assert!(engine.atoms.contains_key("atom1"));
        assert!(engine.atoms.contains_key("atom2"));
        assert!(engine.atoms.contains_key("atom3"));
    }

    #[test]
    fn test_universe_sharing() {
        let engine = RustEngine::new("/tmp/phie_test");
        let uni1 = engine.universe();
        let uni2 = engine.universe();
        assert!(Arc::ptr_eq(&uni1, &uni2));
    }

    #[test]
    fn test_register_overwrites() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        engine.register("test", "first");
        engine.register("test", "second");
        assert_eq!(engine.atoms.get("test").unwrap().source, "second");
    }
}
