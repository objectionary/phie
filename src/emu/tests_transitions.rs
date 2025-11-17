// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

#[cfg(test)]
use crate::basket::Basket;

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use crate::emu::Emu;

#[cfg(test)]
use crate::perf::Perf;

#[test]
pub fn deletes_one_basket() {
    let mut emu = Emu::empty();
    let bk = 1;
    emu.inject(bk, Basket::from_str("[ν1, ξ:β1, 𝜑⇶0x002A]").unwrap());
    let mut perf = Perf::new();
    emu.delete(&mut perf, bk);
    assert!(emu.basket(bk).is_empty())
}

#[test]
fn test_copy_with_delta() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {Δ ⤍ 0x002A}").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→]").unwrap());
    let mut perf = Perf::new();
    emu.copy(&mut perf, 1);
    assert!(matches!(emu.basket(1).kids.get(&crate::loc::Loc::Phi), Some(crate::basket::Kid::Dtzd(42))));
}

#[test]
fn test_copy_without_delta() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {}").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→]").unwrap());
    let mut perf = Perf::new();
    emu.copy(&mut perf, 1);
    assert!(matches!(emu.basket(1).kids.get(&crate::loc::Loc::Phi), Some(crate::basket::Kid::Rqtd)));
}

#[test]
fn test_delete_root_basket() {
    let mut emu = Emu::empty();
    let mut perf = Perf::new();
    emu.delete(&mut perf, 0);
    assert!(!emu.basket(0).is_empty());
}

#[test]
fn test_delete_constant_object() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {Δ ⤍ 0x002A}⤍ Φ̇").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑⇶0x002A]").unwrap());
    let mut perf = Perf::new();
    emu.delete(&mut perf, 1);
    assert!(!emu.basket(1).is_empty());
}

#[test]
fn test_delete_with_waiting_baskets() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {a ⤍ ν2}\nν2 {Δ ⤍ 0x002A}").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.baskets[1].put(crate::loc::Loc::from_str("a").unwrap(), crate::basket::Kid::Wait(2, crate::loc::Loc::Phi));
    let mut perf = Perf::new();
    emu.delete(&mut perf, 2);
    assert!(!emu.basket(2).is_empty());
}

#[test]
fn test_propagate_value() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {a ⤍ ν2}\nν2 {Δ ⤍ 0x002A}").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.baskets[1].put(crate::loc::Loc::from_str("a").unwrap(), crate::basket::Kid::Wait(2, crate::loc::Loc::Phi));
    let mut perf = Perf::new();
    emu.propagate(&mut perf, 2, crate::loc::Loc::Phi);
    assert!(matches!(emu.basket(1).kids.get(&crate::loc::Loc::from_str("a").unwrap()), Some(crate::basket::Kid::Dtzd(42))));
}

#[test]
fn test_delegate_with_lambda() {
    use crate::atom;
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇").unwrap();
    emu.objects[1] = crate::object::Object::lambda("test", atom::int_add);
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→, α0⇶0x0001, α1⇶0x0002]").unwrap());
    let mut perf = Perf::new();
    emu.delegate(&mut perf, 1);
    assert!(matches!(emu.basket(1).kids.get(&crate::loc::Loc::Phi), Some(crate::basket::Kid::Dtzd(3))));
}

#[test]
fn test_delegate_without_lambda() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {}").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→]").unwrap());
    let mut perf = Perf::new();
    emu.delegate(&mut perf, 1);
    assert!(matches!(emu.basket(1).kids.get(&crate::loc::Loc::Phi), Some(crate::basket::Kid::Rqtd)));
}

#[test]
fn test_find_attribute() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {a ⤍ ν2}\nν2 {Δ ⤍ 0x002A}").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, a→]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("a").unwrap());
    assert!(matches!(emu.basket(1).kids.get(&crate::loc::Loc::from_str("a").unwrap()), Some(crate::basket::Kid::Need(_, _))));
}

#[test]
fn test_new_basket() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {a ⤍ ν2}\nν2 {Δ ⤍ 0x002A}").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.baskets[1].put(crate::loc::Loc::from_str("a").unwrap(), crate::basket::Kid::Need(2, 0));
    let mut perf = Perf::new();
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("a").unwrap());
    assert!(matches!(emu.basket(1).kids.get(&crate::loc::Loc::from_str("a").unwrap()), Some(crate::basket::Kid::Wait(_, _))));
}

#[test]
fn test_stashed_finds_constant() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {Δ ⤍ 0x002A}⤍ Φ̇").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑⇶0x002A]").unwrap());
    let result = emu.stashed(1, 0);
    assert_eq!(result, Some(1));
}

#[test]
fn test_stashed_not_found() {
    let mut emu = Emu::from_str("ν0 {}⤍ Φ̇\nν1 {}").unwrap();
    let result = emu.stashed(1, 0);
    assert_eq!(result, None);
}
