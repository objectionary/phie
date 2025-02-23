// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::basket::Bk;
use crate::data::Data;
use crate::emu::Emu;
use crate::loc::Loc;

pub type Atom = fn(&mut Emu, Bk) -> Option<Data>;

pub fn int_add(emu: &mut Emu, bk: Bk) -> Option<Data> {
    Some(emu.read(bk, Loc::Rho)? + emu.read(bk, Loc::Attr(0))?)
}

pub fn int_times(emu: &mut Emu, bk: Bk) -> Option<Data> {
    Some(emu.read(bk, Loc::Rho)? * emu.read(bk, Loc::Attr(0))?)
}

pub fn int_neg(emu: &mut Emu, bk: Bk) -> Option<Data> {
    Some(-emu.read(bk, Loc::Rho)?)
}

pub fn int_sub(emu: &mut Emu, bk: Bk) -> Option<Data> {
    Some(emu.read(bk, Loc::Rho)? - emu.read(bk, Loc::Attr(0))?)
}

pub fn int_div(emu: &mut Emu, bk: Bk) -> Option<Data> {
    Some(emu.read(bk, Loc::Rho)? / emu.read(bk, Loc::Attr(0))?)
}

pub fn int_less(emu: &mut Emu, bk: Bk) -> Option<Data> {
    Some((emu.read(bk, Loc::Rho)? < emu.read(bk, Loc::Attr(0))?) as Data)
}

pub fn bool_if(emu: &mut Emu, bk: Bk) -> Option<Data> {
    let term = emu.read(bk, Loc::Rho)?;
    emu.read(bk, Loc::Attr(if term == 1 { 0 } else { 1 }))
}

#[cfg(test)]
use crate::assert_dataized_eq;

#[cfg(test)]
use crate::emu::Opt;

#[test]
pub fn bool_if_works() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ bool-if, ρ ↦ ν1, 𝛼0 ↦ ν3, 𝛼1 ↦ ν4 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν4(𝜋) ↦ ⟦ Δ ↦ 0x0000 ⟧
    "
    );
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x0000 ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ bool-if, ρ ↦ ν1, 𝛼0 ↦ ν3, 𝛼1 ↦ ν4 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0000 ⟧
        ν4(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
    "
    );
}

#[test]
pub fn int_add_works() {
    assert_dataized_eq!(
        49,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x0007 ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν1, 𝛼0 ↦ ν3 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
    "
    );
}

#[test]
pub fn int_times_works() {
    assert_dataized_eq!(
        77,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x0007 ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-times, ρ ↦ ν1, 𝛼0 ↦ ν3 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x000B ⟧
    "
    );
}

#[test]
pub fn int_sub_works() {
    assert_dataized_eq!(
        40,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-sub, ρ ↦ ν1, 𝛼0 ↦ ν3 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧
    "
    );
}

#[test]
pub fn int_div_works() {
    assert_dataized_eq!(
        21,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-div, ρ ↦ ν1, 𝛼0 ↦ ν3 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧
    "
    );
}

#[test]
pub fn int_less_works() {
    assert_dataized_eq!(
        0,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-less, ρ ↦ ν1, 𝛼0 ↦ ν3 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧
    "
    );
    assert_dataized_eq!(
        0,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-less, ρ ↦ ν1, 𝛼0 ↦ ν3 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
    "
    );
    assert_dataized_eq!(
        1,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2 ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-less, ρ ↦ ν1, 𝛼0 ↦ ν3 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x002B ⟧
    "
    );
}
