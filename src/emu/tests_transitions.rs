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
