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

extern crate phie;

use phie::data::Data;
use phie::emu::{Emu, Opt};
use std::env;

pub fn fibo(x: Data) -> Data {
    let mut emu: Emu = format!(
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
        x
    )
    .parse()
    .unwrap();
    emu.opt(Opt::LogSnapshots);
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    emu.dataize().0
}

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let input = args[1].parse().unwrap();
    let cycles = args[2].parse().unwrap();
    let mut f = 0;
    for _ in 0..cycles {
        f = fibo(input);
    }
    println!("{}-th Fibonacci number is {}", input, f);
}

#[cfg(test)]
use simple_logger::SimpleLogger;

#[test]
fn calculates_fibonacci() {
    SimpleLogger::new().init().unwrap();
    assert_eq!(21, fibo(7))
}
