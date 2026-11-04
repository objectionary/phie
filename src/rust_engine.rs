// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! Rust atom execution engine wired into the emulator.
//!
//! The engine owns the lifecycle of every Rust atom attached to an [`Emu`]:
//! registration of sources, lazy compilation to shared libraries, and
//! execution against a [`Universe`] that mirrors the dataized attributes
//! of the atom object as SODG vertices.
//!
//! [`rust_atom`] is the bridge function registered for the `λ ↦ rust`
//! lambda: the emulator's `delegate()` transition calls it whenever a
//! basket of a Rust atom object is asked for its `𝜑`.

use crate::basket::Bk;
use crate::data::Data;
use crate::emu::Emu;
use crate::loc::Loc;
use crate::object::Ob;
use crate::rust_atom::RustAtom;
use crate::universe::{Universe, VertexId};
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;

/// Manages the Rust atoms of a single emulator run.
///
/// Atoms are keyed by the object number (`ν`) they belong to. The engine
/// compiles each atom once, on first execution, and reuses the loaded
/// library for later calls.
///
/// # Examples
///
/// ```
/// use phie::rust_engine::RustEngine;
///
/// let engine = RustEngine::new("/tmp/phie-engine-example");
/// assert!(!engine.contains(0));
/// ```
pub struct RustEngine {
    /// Registered atoms keyed by the object number they implement.
    atoms: HashMap<Ob, RustAtom>,
    /// Directory for build artifacts and compilation output.
    build_dir: PathBuf,
    /// The graph every atom of this run shares.
    universe: Universe,
}

impl RustEngine {
    /// Creates a new engine with a build directory.
    ///
    /// # Arguments
    ///
    /// * `build_dir` - Directory for build artifacts
    pub fn new(build_dir: &str) -> Self {
        RustEngine {
            atoms: HashMap::new(),
            build_dir: PathBuf::from(build_dir),
            universe: Universe::new(),
        }
    }

    /// FNV-1a hash of the source, used to keep the build directory of an
    /// atom unique to its content, so a re-registered object with a new
    /// source never reuses a stale library and an unchanged one does.
    fn source_hash(source: &str) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in source.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }

    /// Registers the source of the Rust atom of object `ν{ob}`.
    ///
    /// # Arguments
    ///
    /// * `ob` - Object number the atom implements
    /// * `source` - Rust source code defining `pub fn atom`
    pub fn register(&mut self, ob: Ob, source: &str) {
        let id = format!("atom_{ob}_{:x}", Self::source_hash(source));
        self.atoms.insert(ob, RustAtom::new(&id, source));
    }

    /// Tells whether object `ν{ob}` has a registered Rust atom.
    pub fn contains(&self, ob: Ob) -> bool {
        self.atoms.contains_key(&ob)
    }

    /// Compiles the atom of object `ν{ob}`, unless already compiled.
    pub fn compile(&mut self, ob: Ob) -> Result<(), String> {
        let atom = self
            .atoms
            .get_mut(&ob)
            .ok_or_else(|| format!("Atom at ν{ob} is not registered"))?;
        if atom.is_compiled() {
            return Ok(());
        }
        let dir = self
            .build_dir
            .to_str()
            .ok_or_else(|| "Build dir is not valid UTF-8".to_string())?;
        atom.compile(dir)?;
        Ok(())
    }

    /// Executes the compiled atom of object `ν{ob}` against a vertex.
    pub fn execute(&mut self, ob: Ob, vertex: VertexId) -> Result<Data, String> {
        let atom = self
            .atoms
            .get(&ob)
            .ok_or_else(|| format!("Atom at ν{ob} is not registered"))?;
        atom.execute(&mut self.universe, vertex)
    }

    /// Returns the universe shared by all atoms of this engine.
    pub fn universe_mut(&mut self) -> &mut Universe {
        &mut self.universe
    }
}

/// The `rust` atom: dataizes an object backed by a compiled Rust atom.
///
/// Returns `None` while the engine, the registration, or any `𝛼` of the
/// object is not ready yet, exactly like the built-in atoms do. Once every
/// `𝛼` carries data, the engine seeds a fresh vertex of the [`Universe`]
/// with one labelled edge per `𝛼`, compiles the atom if needed, and calls
/// it; the atom then reads its arguments through `uni.dataize("vN.𝛼0")`.
///
/// # Panics
///
/// Panics with a descriptive message when compilation or execution of a
/// ready atom fails, the same way the emulator panics on unresolvable
/// states.
///
/// # Examples
///
/// ```no_run
/// use phie::emu::{Emu, Opt};
///
/// let source = "pub fn atom(uni: &Universe, v: u32) -> Result<i16, String> {\n    Ok(uni.dataize(&format!(\"v{v}.\\u{1d6fc}0\"))?)\n}";
/// let mut emu: Emu = "
///     ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
///     ν2(𝜋) ↦ ⟦ λ ↦ rust, 𝛼0 ↦ ν3(𝜋) ⟧
///     ν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
/// "
/// .parse()
/// .unwrap();
/// emu.register_rust_atom(2, source);
/// emu.opt(Opt::StopWhenTooManyCycles);
/// assert_eq!(42, emu.dataize().0);
/// ```
pub fn rust_atom(emu: &mut Emu, bk: Bk) -> Option<Data> {
    let ob = emu.basket(bk).ob;
    if !emu.rust.as_ref().is_some_and(|engine| engine.contains(ob)) {
        return None;
    }
    let attrs: Vec<i8> = emu
        .object(ob)
        .attrs
        .keys()
        .filter_map(|loc| match loc {
            Loc::Attr(i) => Some(*i),
            _ => None,
        })
        .collect();
    let mut args: Vec<(i8, Data)> = Vec::with_capacity(attrs.len());
    for i in attrs {
        args.push((i, emu.read(bk, Loc::Attr(i))?));
    }
    let engine = emu.rust.as_mut().unwrap_or_else(|| {
        panic!("The Rust engine vanished while dataizing ν{ob}");
    });
    engine
        .compile(ob)
        .unwrap_or_else(|e| panic!("Can't compile the Rust atom of ν{ob}: {e}"));
    let vertex = seed(engine, ob, &args);
    debug!("rust_atom(β{}/ν{ob}): executed on v{vertex}", bk);
    Some(
        engine
            .execute(ob, vertex)
            .unwrap_or_else(|e| panic!("Can't execute the Rust atom of ν{ob}: {e}")),
    )
}

/// Mirrors the dataized attributes of the atom object into the graph.
///
/// Creates a vertex for the atom itself and one vertex per `𝛼` carrying
/// its data, bound to the atom vertex by the `𝛼N` label, which is the
/// shape `uni.dataize("vN.𝛼0")` expects.
fn seed(engine: &mut RustEngine, ob: Ob, args: &[(i8, Data)]) -> VertexId {
    let universe = engine.universe_mut();
    let vertex = universe.next_id();
    universe
        .add(vertex)
        .unwrap_or_else(|e| panic!("Can't add the atom vertex of ν{ob}: {e}"));
    for (i, data) in args {
        let kid = universe.next_id();
        universe
            .add(kid)
            .unwrap_or_else(|e| panic!("Can't add the vertex of 𝛼{i} of ν{ob}: {e}"));
        universe
            .put(kid, *data)
            .unwrap_or_else(|e| panic!("Can't put the data of 𝛼{i} of ν{ob}: {e}"));
        universe
            .bind(vertex, kid, &format!("𝛼{i}"))
            .unwrap_or_else(|e| panic!("Can't bind 𝛼{i} of ν{ob}: {e}"));
    }
    vertex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = RustEngine::new("/tmp/phie_test");
        assert!(engine.atoms.is_empty());
        assert!(!engine.contains(0));
    }

    #[test]
    fn test_register_atom() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        engine.register(0, "fn main() {}");
        assert!(engine.contains(0));
        assert!(!engine.contains(1));
    }

    #[test]
    fn test_register_overwrites() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        engine.register(0, "first");
        engine.register(0, "second");
        assert_eq!(engine.atoms.get(&0).unwrap().source, "second");
    }

    #[test]
    fn test_compile_nonexistent_atom() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        let err = engine.compile(7).expect_err("must fail");
        assert!(err.contains("not registered"), "{err}");
    }

    #[test]
    fn test_execute_nonexistent_atom() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        let err = engine.execute(7, 0).expect_err("must fail");
        assert!(err.contains("not registered"), "{err}");
    }

    #[test]
    fn test_execute_uncompiled_atom() {
        let mut engine = RustEngine::new("/tmp/phie_test");
        engine.register(0, "fn test() {}");
        assert!(engine.execute(0, 0).is_err());
    }

    #[test]
    fn test_compile_and_execute() {
        let mut engine = RustEngine::new(
            std::env::temp_dir()
                .join("phie_engine_flow")
                .to_str()
                .unwrap(),
        );
        engine.register(
            0,
            "pub fn atom(uni: &Universe, v: u32) -> Result<i16, String> {\n    Ok(uni.dataize(&format!(\"v{v}.\\u{1d6fc}0\"))?)\n}",
        );
        engine.compile(0).expect("must compile");
        engine.compile(0).expect("second compile is a no-op");
        let universe = engine.universe_mut();
        let v = universe.next_id();
        let kid = universe.next_id();
        universe.add(v).unwrap();
        universe.add(kid).unwrap();
        universe.put(kid, 42).unwrap();
        universe.bind(v, kid, "𝛼0").unwrap();
        assert_eq!(42, engine.execute(0, v).unwrap());
        std::fs::remove_dir_all(std::env::temp_dir().join("phie_engine_flow")).unwrap();
    }

    #[test]
    fn test_seed_mirrors_args_into_graph() {
        let mut engine = RustEngine::new("/tmp/phie_seed_test");
        let vertex = seed(&mut engine, 5, &[(0, 4), (1, 8)]);
        let universe = engine.universe_mut();
        assert_eq!(
            4,
            universe.dataize(&format!("v{vertex}.\u{1d6fc}0")).unwrap()
        );
        assert_eq!(
            8,
            universe.dataize(&format!("v{vertex}.\u{1d6fc}1")).unwrap()
        );
    }
}
