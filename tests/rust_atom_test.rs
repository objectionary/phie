// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! End-to-end tests: a program with a Rust atom runs through the emulator.

use phie::emu::{Emu, Opt};

#[test]
fn dataizes_sum_rust_atom() {
    let source = r#"
pub fn atom(uni: &Universe, v: u32) -> Result<i16, String> {
    let a = uni.dataize(&format!("v{v}.\u{1d6fc}0"))?;
    let b = uni.dataize(&format!("v{v}.\u{1d6fc}1"))?;
    Ok(a + b)
}
"#;
    let mut emu: Emu = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ rust, 𝛼0 ↦ ν3(𝜋), 𝛼1 ↦ ν4(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0004 ⟧
        ν4(𝜋) ↦ ⟦ Δ ↦ 0x0008 ⟧
    "
    .parse()
    .expect("the program must parse");
    emu.register_rust_atom(2, source);
    emu.opt(Opt::StopWhenTooManyCycles);
    assert_eq!(12, emu.dataize().0);
}

#[test]
fn dataizes_rust_atom_reading_one_attribute() {
    let source = r#"
pub fn atom(uni: &Universe, v: u32) -> Result<i16, String> {
    uni.dataize(&format!("v{v}.\u{1d6fc}0"))
}
"#;
    let mut emu: Emu = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ rust, 𝛼0 ↦ ν3(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
    "
    .parse()
    .expect("the program must parse");
    emu.register_rust_atom(2, source);
    emu.opt(Opt::StopWhenTooManyCycles);
    assert_eq!(42, emu.dataize().0);
}

#[test]
fn dataizes_nested_rust_atom_through_int_add() {
    let source = r#"
pub fn atom(uni: &Universe, v: u32) -> Result<i16, String> {
    let a = uni.dataize(&format!("v{v}.\u{1d6fc}0"))?;
    Ok(a * 2)
}
"#;
    let mut emu: Emu = "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν3 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x0005 ⟧
        ν2(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν3(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν1(𝜋), 𝛼0 ↦ ν4(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ λ ↦ rust, 𝛼0 ↦ ν2(𝜋) ⟧
    "
    .parse()
    .expect("the program must parse");
    emu.register_rust_atom(4, source);
    emu.opt(Opt::StopWhenTooManyCycles);
    assert_eq!(7, emu.dataize().0);
}
