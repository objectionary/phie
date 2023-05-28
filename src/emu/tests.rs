// Copyright (c) 2022 Yegor Bugayenko
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[cfg(test)]
use crate::emu::{Emu, Opt};

#[cfg(test)]
use crate::perf::Transition;

#[cfg(test)]
use crate::loc::Loc;

#[cfg(test)]
use crate::locator::Locator;

#[cfg(test)]
use crate::data::Data;

#[cfg(test)]
use crate::ph;

#[cfg(test)]
use crate::assert_dataized_eq;

#[cfg(test)]
use crate::object::Object;

#[cfg(test)]
use std::str::FromStr;

#[test]
pub fn simple_dataization_cycle() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("ν1"), true));
    emu.put(1, Object::dataic(42));
    assert_eq!(42, emu.dataize().0);
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("ν2"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("ν1"), false));
    assert_eq!(42, emu.dataize().0);
}

#[test]
pub fn with_many_decorators() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("ν4"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("ν1"), false));
    emu.put(3, Object::open().with(Loc::Phi, ph!("ν2"), false));
    emu.put(4, Object::open().with(Loc::Phi, ph!("ν3"), false));
    assert_eq!(42, emu.dataize().0);
}

// []
//   42 > x
//   42 > y
//   int-add > @
//     $.x
//     $.y
#[test]
pub fn summarizes_two_numbers() {
    assert_dataized_eq!(
        84,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν3(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ 𝜋.𝛼1 ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼0 ↦ ν1(𝜋), 𝛼1 ↦ ν1(𝜋) ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ) ⟧
        "
    );
}

// []
//   int-add > @    v1
//     int-add      v2
//       42         v9
//       42         v9
//     int-add      v3
//       int-neg    v4
//         42       v9
//       42         v9
#[test]
pub fn preserves_calculation_results() {
    let mut emu = Emu::from_str(
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν9(𝜋), 𝛼0 ↦ ν9(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν4(𝜋), 𝛼0 ↦ ν9(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ λ ↦ int-neg, ρ ↦ ν9(𝜋) ⟧
        ν9(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ",
    )
    .unwrap();
    let dtz = emu.dataize();
    assert_eq!(84, dtz.0);
    let perf = dtz.1;
    assert_eq!(4, perf.total_atoms());
}

// []
//   foo > @        v1
//     int-add      v2
//       42         v9
//       42         v9
// [x] > foo        v3
//   int-add        v4
//     $.x
//     42           v9
#[test]
pub fn calculates_argument_once() {
    let mut emu = Emu::from_str(
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν1(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν9(𝜋), 𝛼0 ↦ ν9(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν4(𝜋), 𝛼0 ↦ ν9(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ λ ↦ int-neg, ρ ↦ ν9(𝜋) ⟧
        ν9(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ",
    )
    .unwrap();
    let dtz = emu.dataize();
    assert_eq!(84, dtz.0);
    let perf = dtz.1;
    assert_eq!(4, perf.total_atoms());
}

// []
//   int-add > x!          v1
//     2                   v2
//     3                   v3
//   int-add > @           v4
//     x
//     x
#[test]
pub fn summarizes_two_pairs_of_numbers() {
    assert_dataized_eq!(
        10,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν4(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν2(𝜋), 𝛼0 ↦ ν3(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧
        ν3(𝜋) ↦ ⟦ Δ ↦ 0x0003 ⟧
        ν4(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν1(𝜋), 𝛼0 ↦ ν1(𝜋) ⟧
        "
    );
}

// [x] > a
//   $.x > @
// a > foo
//   a 42 > @
#[test]
pub fn calls_itself_once() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν4(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν2(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν2(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν3(𝜋) ⟧
        "
    );
}

// [x] > a
//   $.x > @
// [y] > b
//   a > @
//     $.y
// b 42 > foo
#[test]
pub fn injects_xi_correctly() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν5(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ) ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ 𝜋.𝛼0 ⟧
        ν4(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼0 ↦ ν4(𝜋) ⟧
        "
    );
}

// [a3] > v1         v1
//   $.a3 > @
// [a1] > v2         v2
//   v1 > @          v3
//     $.a1
// v2 42 > @         v4
#[test]
pub fn reverse_to_abstract() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν3(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼3 ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼3 ↦ 𝜋.𝛼1 ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼1 ↦ ν4(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a          v1
//   b > @          v2
//     c            v3
//       $.x
// [x] > b          v4
//   x > @
// [x] > c          v5
//   x > @
// a                v6
//   42             v7
#[test]
pub fn passes_xi_through_two_layers() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν6(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν4(ξ), 𝛼0 ↦ ν3(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν5(ξ), 𝛼0 ↦ 𝜋.𝜋.𝛼0 ⟧
        ν4(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν6(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν7(𝜋) ⟧
        ν7(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a          v1
//   b > @          v2
//     c            v3
//       d          v4
//         $.x
// [x] > b          v5
//   x > @
// [x] > c          v6
//   x > @
// [x] > d          v7
//   x > @
// a                v8
//   42             v9
#[test]
pub fn passes_xi_through_three_layers() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν8(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν5(ξ), 𝛼0 ↦ ν3(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν6(ξ), 𝛼0 ↦ ν4(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ 𝜑 ↦ ν7(ξ), 𝛼0 ↦ 𝜋.𝜋.𝜋.𝛼0 ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν6(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν7(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν8(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν9(𝜋) ⟧
        ν9(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a          v1
//   b > @          v2
//     c            v3
//       d          v4
//         e        v5
//           $.x
// [x] > b          v6
//   x > @
// [x] > c          v7
//   x > @
// [x] > d          v8
//   x > @
// [x] > e          v9
//   x > @
// a                v10
//   42             v11
#[test]
pub fn passes_xi_through_four_layers() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν10(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν6(ξ), 𝛼0 ↦ ν3(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν7(ξ), 𝛼0 ↦ ν4(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ 𝜑 ↦ ν8(ξ), 𝛼0 ↦ ν5(𝜋) ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ ν9(ξ), 𝛼0 ↦ 𝜋.𝜋.𝜋.𝜋.𝛼0 ⟧
        ν6(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν7(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν8(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν9(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν10(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν11(𝜋) ⟧
        ν11(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a        v1
//   b > @        v2
//     c          v3
//       $.x
// [x] > b        v4
//   c > @        v5
//     $.x
// [x] > c        v6
//   x > @
// a              v7
//   42           v8
#[test]
pub fn simulation_of_recursion() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν7(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν4(ξ), 𝛼0 ↦ ν3(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν6(ξ), 𝛼0 ↦ 𝜋.𝜋.𝛼0 ⟧
        ν4(𝜋) ↦ ⟦ 𝜑 ↦ ν5(𝜋) ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ ν6(ξ), 𝛼0 ↦ 𝜋.𝛼0 ⟧
        ν6(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν7(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν8(𝜋) ⟧
        ν8(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a        v1
//   b > @        v2
//     f          v3
//       $.x
// [x] > b        v4
//   c > @        v5
//     f          v6
//       $.x
// [x] > c        v7
//   f > @        v8
//     $.x
// [x] > f        v9
//   x > @
// a              v10
//   42           v11
#[test]
pub fn deep_simulation_of_recursion() {
    assert_dataized_eq!(
        42,
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν10(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν4(ξ), 𝛼0 ↦ ν3(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν9(ξ), 𝛼0 ↦ 𝜋.𝜋.𝛼0 ⟧
        ν4(𝜋) ↦ ⟦ 𝜑 ↦ ν5(𝜋) ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ ν7(ξ), 𝛼0 ↦ ν6(𝜋) ⟧
        ν6(𝜋) ↦ ⟦ 𝜑 ↦ ν9(ξ), 𝛼0 ↦ 𝜋.𝜋.𝛼0 ⟧
        ν7(𝜋) ↦ ⟦ 𝜑 ↦ ν8(𝜋) ⟧
        ν8(𝜋) ↦ ⟦ 𝜑 ↦ ν9(ξ), 𝛼0 ↦ 𝜋.𝛼0 ⟧
        ν9(𝜋) ↦ ⟦ 𝜑 ↦ 𝜋.𝛼0 ⟧
        ν10(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν11(𝜋) ⟧
        ν11(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > foo        v1
//   bool-if        v2
//     int-less     v3
//       $.x
//       0          v4
//     42           v5
//     foo          v6
//       int-sub    v7
//         $.x
//         1        v8
// foo              v9
//   7              v10
#[test]
pub fn simple_recursion() {
    let mut emu = Emu::from_str(
        "
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν9(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ bool-if, ρ ↦ ν3(𝜋), 𝛼0 ↦ ν5(𝜋), 𝛼1 ↦ ν6(𝜋) ⟧
        ν3(𝜋) ↦ ⟦ λ ↦ int-less, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ ν4(𝜋) ⟧
        ν4(𝜋) ↦ ⟦ Δ ↦ 0x0000 ⟧
        ν5(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν6(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν7(𝜋) ⟧
        ν7(𝜋) ↦ ⟦ λ ↦ int-sub, ρ ↦ 𝜋.𝜋.𝛼0, 𝛼0 ↦ ν8(𝜋) ⟧
        ν8(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν9(𝜋) ↦ ⟦ 𝜑 ↦ ν1(ξ), 𝛼0 ↦ ν10(𝜋) ⟧
        ν10(𝜋) ↦ ⟦ Δ ↦ 0x0007 ⟧
        ",
    )
    .unwrap();
    emu.opt(Opt::DontDelete);
    let dtz = emu.dataize();
    let perf = dtz.1;
    assert_eq!(9, emu.baskets.iter().filter(|bsk| bsk.ob == 1).count());
    assert_eq!(4, *perf.hits.get(&Transition::CPY).unwrap());
}

#[cfg(test)]
fn fibo(n: Data) -> Data {
    if n < 2 {
        return 1;
    }
    fibo(n - 1) + fibo(n - 2)
}

#[cfg(test)]
fn fibo_ops(n: Data) -> usize {
    if n < 2 {
        return 2;
    }
    fibo_ops(n - 1) + fibo_ops(n - 2) + 5
}

#[test]
pub fn recursive_fibonacci() {
    let input = 7;
    let mut emu = Emu::from_str(
        format!(
            "
            ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν2(𝜋) ⟧
            ν1(𝜋) ↦ ⟦ Δ ↦ 0x{:04X} ⟧
            ν2(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ), 𝛼0 ↦ ν1(𝜋) ⟧
            ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν13(𝜋) ⟧
            ν5(𝜋) ↦ ⟦ Δ ↦ 0x0002 ⟧
            ν6(𝜋) ↦ ⟦ λ ↦ int-sub, ρ ↦ 𝜋.𝜋.𝛼0, 𝛼0 ↦ ν5(𝜋) ⟧
            ν7(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧
            ν8(𝜋) ↦ ⟦ λ ↦ int-sub, ρ ↦ 𝜋.𝜋.𝛼0, 𝛼0 ↦ ν7(𝜋) ⟧
            ν9(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ), 𝛼0 ↦ ν8(𝜋) ⟧
            ν10(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ), 𝛼0 ↦ ν6(𝜋) ⟧
            ν11(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν9(𝜋), 𝛼0 ↦ ν10(𝜋) ⟧
            ν12(𝜋) ↦ ⟦ λ ↦ int-less, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ ν5(𝜋) ⟧
            ν13(𝜋) ↦ ⟦ λ ↦ bool-if, ρ ↦ ν12(𝜋), 𝛼0 ↦ ν7(𝜋), 𝛼1 ↦ ν11(𝜋) ⟧
            ",
            input
        )
        .as_str(),
    )
    .unwrap();
    let dtz = emu.dataize();
    assert_eq!(fibo(input), dtz.0, "Wrong number calculated");
    let perf = dtz.1;
    assert_eq!(
        perf.total_atoms(),
        fibo_ops(input),
        "Too many atomic operations"
    );
}
