// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use crate::basket::Basket;
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
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→?]").unwrap());
    let mut perf = Perf::new();
    emu.copy(&mut perf, 1);
    assert!(matches!(
        emu.basket(1).kids.get(&crate::loc::Loc::Phi),
        Some(crate::basket::Kid::Dtzd(42))
    ));
}

#[test]
fn test_copy_without_delta() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→?]").unwrap());
    let mut perf = Perf::new();
    emu.copy(&mut perf, 1);
    assert!(matches!(
        emu.basket(1).kids.get(&crate::loc::Loc::Phi),
        Some(crate::basket::Kid::Rqtd)
    ));
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
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧").unwrap();
    emu.objects[1].constant = true;
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑⇶0x002A]").unwrap());
    let mut perf = Perf::new();
    emu.delete(&mut perf, 1);
    assert!(!emu.basket(1).is_empty());
}

#[test]
fn test_delete_with_waiting_baskets() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Wait(2, crate::loc::Loc::Phi)
    );
    let mut perf = Perf::new();
    emu.delete(&mut perf, 2);
    assert!(!emu.basket(2).is_empty());
}

#[test]
fn test_propagate_value() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Wait(2, crate::loc::Loc::Phi)
    );
    let mut perf = Perf::new();
    emu.propagate(&mut perf, 2, crate::loc::Loc::Phi);
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Dtzd(42))
    ));
}

#[test]
fn test_delegate_with_lambda() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧\nν1(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧\nν3(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧")
        .unwrap();
    emu.inject(
        1,
        Basket::from_str("[ν1, ξ:β0, 𝜑→?, ρ⇶0x0001, 𝛼0⇶0x0002]").unwrap()
    );
    let mut perf = Perf::new();
    emu.delegate(&mut perf, 1);
    assert!(matches!(
        emu.basket(1).kids.get(&crate::loc::Loc::Phi),
        Some(crate::basket::Kid::Dtzd(3))
    ));
}

#[test]
fn test_delegate_without_lambda() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→?]").unwrap());
    let mut perf = Perf::new();
    emu.delegate(&mut perf, 1);
    assert!(matches!(
        emu.basket(1).kids.get(&crate::loc::Loc::Phi),
        Some(crate::basket::Kid::Rqtd)
    ));
}

#[test]
fn test_find_attribute() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Need(_, _))
    ));
}

#[test]
fn test_new_basket() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    let mut perf = Perf::new();
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Wait(_, _))
    ));
}

#[test]
fn test_new_finds_stashed_constant() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.objects[2].constant = true;
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    let mut perf = Perf::new();
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Wait(2, _))
    ));
}

#[test]
fn test_copy_with_non_rqtd_phi() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑⇶0x002A]").unwrap());
    let mut perf = Perf::new();
    emu.copy(&mut perf, 1);
    assert!(matches!(
        emu.basket(1).kids.get(&crate::loc::Loc::Phi),
        Some(crate::basket::Kid::Dtzd(42))
    ));
}

#[test]
fn test_propagate_with_non_dtzd_loc() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑→?]").unwrap());
    let mut perf = Perf::new();
    emu.propagate(&mut perf, 2, crate::loc::Loc::Phi);
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        None
    ));
}

#[test]
fn test_delete_with_empt_kid() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Empt
    );
    let mut perf = Perf::new();
    emu.delete(&mut perf, 1);
    assert!(emu.basket(1).is_empty());
}

#[test]
fn test_delete_with_rqtd_kid() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑⇶0x002A, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.delete(&mut perf, 1);
    assert!(!emu.basket(1).is_empty());
}

#[test]
fn test_delegate_with_wait_in_basket() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧\nν1(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧\nν3(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧")
        .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→?]").unwrap());
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑⇶0x0001]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::Rho,
        crate::basket::Kid::Wait(2, crate::loc::Loc::Phi)
    );
    let mut perf = Perf::new();
    emu.delegate(&mut perf, 1);
    assert!(matches!(
        emu.basket(1).kids.get(&crate::loc::Loc::Phi),
        Some(crate::basket::Kid::Rqtd)
    ));
}

#[test]
fn test_find_with_non_rqtd_loc() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0⇶0x002A]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Dtzd(42))
    ));
}

#[test]
fn test_find_with_advice_false() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ 𝜋.𝛼1 ⟧\nν2(𝜋) ↦ ⟦ 𝛼1 ↦ ν3(𝜋) ⟧\nν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
        .unwrap();
    emu.inject(2, Basket::from_str("[ν2, ξ:β0]").unwrap());
    emu.inject(1, Basket::from_str("[ν1, ξ:β2, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Wait(_, _))
    ));
}

#[test]
fn test_find_with_non_empt_ploc() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ 𝜋.𝛼1 ⟧\nν2(𝜋) ↦ ⟦ 𝛼1 ↦ ν3(𝜋) ⟧\nν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
        .unwrap();
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝛼1⇶0x002A]").unwrap());
    emu.inject(1, Basket::from_str("[ν1, ξ:β2, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Wait(_, _))
    ));
}

#[test]
fn test_new_with_non_need_loc() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Rqtd
    );
    let mut perf = Perf::new();
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Rqtd)
    ));
}

#[test]
fn test_stashed_with_delta() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    let mut perf = Perf::new();
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Wait(2, _))
    ));
}

#[test]
fn test_stashed_non_constant() {
    let mut emu = Emu::from_str(
        "ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋), 𝛼1 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧"
    )
    .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼1").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    let mut perf = Perf::new();
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼1").unwrap());
    let bk0 = if let Some(crate::basket::Kid::Wait(b, _)) = emu
        .basket(1)
        .kids
        .get(&crate::loc::Loc::from_str("𝛼0").unwrap())
    {
        *b
    } else {
        panic!()
    };
    let bk1 = if let Some(crate::basket::Kid::Wait(b, _)) = emu
        .basket(1)
        .kids
        .get(&crate::loc::Loc::from_str("𝛼1").unwrap())
    {
        *b
    } else {
        panic!()
    };
    assert_ne!(bk0, bk1);
}

#[test]
#[should_panic(expected = "Can't find 𝜋")]
fn test_search_pi_on_root() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ 𝜋 ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
}

#[test]
fn test_search_finds_object_without_phi() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2 ⟧\nν2(𝜋) ↦ ⟦ 𝛼1 ↦ ν0(𝜋) ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Need(2, _))
    ));
}

#[test]
fn test_search_with_loc_root() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ Φ ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Need(0, _))
    ));
}

#[test]
fn test_search_with_loc_obj() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2 ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Need(2, _))
    ));
}

#[test]
fn test_search_with_pi() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ 𝜋.𝛼1 ⟧\nν2(𝜋) ↦ ⟦ 𝛼1 ↦ ν3(𝜋) ⟧\nν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
        .unwrap();
    emu.inject(2, Basket::from_str("[ν2, ξ:β0]").unwrap());
    emu.inject(1, Basket::from_str("[ν1, ξ:β2, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Wait(_, _))
    ));
}

#[test]
fn test_find_without_attr() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ Φ ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Need(0, _))
    ));
}

#[test]
fn test_stashed_constant_same_psi() {
    let mut emu = Emu::from_str(
        "ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋), 𝛼1 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧"
    )
    .unwrap();
    emu.objects[2].constant = true;
    emu.inject(1, Basket::from_str("[ν1, ξ:β0]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼1").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    let mut perf = Perf::new();
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    emu.new(&mut perf, 1, crate::loc::Loc::from_str("𝛼1").unwrap());
    let bk0 = if let Some(crate::basket::Kid::Wait(b, _)) = emu
        .basket(1)
        .kids
        .get(&crate::loc::Loc::from_str("𝛼0").unwrap())
    {
        *b
    } else {
        panic!()
    };
    let bk1 = if let Some(crate::basket::Kid::Wait(b, _)) = emu
        .basket(1)
        .kids
        .get(&crate::loc::Loc::from_str("𝛼1").unwrap())
    {
        *b
    } else {
        panic!()
    };
    assert_eq!(bk0, bk1);
}

#[test]
fn test_delegate_returns_none() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧\nν1(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧\nν3(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧")
        .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑→?, ρ→?]").unwrap());
    let mut perf = Perf::new();
    emu.delegate(&mut perf, 1);
    assert!(matches!(
        emu.basket(1).kids.get(&crate::loc::Loc::Phi),
        Some(crate::basket::Kid::Rqtd)
    ));
}

#[test]
fn test_find_attr_not_in_object() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧").unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(1)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼0").unwrap()),
        Some(crate::basket::Kid::Rqtd)
    ));
}

#[test]
fn test_find_with_ploc_empt() {
    let mut emu = Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ 𝜋.𝛼1 ⟧\nν2(𝜋) ↦ ⟦ 𝛼1 ↦ ν3(𝜋) ⟧\nν3(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
        .unwrap();
    emu.inject(2, Basket::from_str("[ν2, ξ:β0, 𝛼1→∅]").unwrap());
    emu.inject(1, Basket::from_str("[ν1, ξ:β2, 𝛼0→?]").unwrap());
    let mut perf = Perf::new();
    emu.find(&mut perf, 1, crate::loc::Loc::from_str("𝛼0").unwrap());
    assert!(matches!(
        emu.basket(2)
            .kids
            .get(&crate::loc::Loc::from_str("𝛼1").unwrap()),
        Some(crate::basket::Kid::Wait(_, _))
    ));
}

#[test]
fn test_delete_with_need_kid() {
    let mut emu =
        Emu::from_str("ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν0(𝜋) ⟧\nν1(𝜋) ↦ ⟦ 𝛼0 ↦ ν2(𝜋) ⟧\nν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧")
            .unwrap();
    emu.inject(1, Basket::from_str("[ν1, ξ:β0, 𝜑⇶0x002A]").unwrap());
    emu.baskets[1].put(
        crate::loc::Loc::from_str("𝛼0").unwrap(),
        crate::basket::Kid::Need(2, 0)
    );
    let mut perf = Perf::new();
    emu.delete(&mut perf, 1);
    assert!(!emu.basket(1).is_empty());
}
