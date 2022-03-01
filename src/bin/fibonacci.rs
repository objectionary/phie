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

extern crate eoc;

use eoc::data::Data;
use eoc::emu::Emu;
use std::env;

pub fn fibo(x: Data) -> Data {
    let mut emu: Emu = format!(
        "
        ν0 ↦ ⟦ φ ↦ ν2 ⟧
        ν1 ↦ ⟦ Δ ↦ 0x{:04X} ⟧
        ν2 ↦ ⟦ φ ↦ ν3(ξ), 𝛼0 ↦ ν1 ⟧
        ν3 ↦ ⟦ φ ↦ ν13 ⟧
        ν5 ↦ ⟦ Δ ↦ 0x0002 ⟧
        ν6 ↦ ⟦ λ ↦ int-sub, ρ ↦ ξ.ξ.𝛼0, 𝛼0 ↦ ν5 ⟧
        ν7 ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν8 ↦ ⟦ λ ↦ int-sub, ρ ↦ ξ.ξ.𝛼0, 𝛼0 ↦ ν7 ⟧
        ν9 ↦ ⟦ φ ↦ ν3(ξ), 𝛼0 ↦ ν8 ⟧
        ν10 ↦ ⟦ φ ↦ ν3(ξ), 𝛼0 ↦ ν6 ⟧
        ν11 ↦ ⟦ λ ↦ int-add, ρ ↦ ν9, 𝛼0 ↦ ν10 ⟧
        ν12 ↦ ⟦ λ ↦ int-less, ρ ↦ ξ.𝛼0, 𝛼0 ↦ ν5 ⟧
        ν13 ↦ ⟦ λ ↦ bool-if, ρ ↦ ν12, 𝛼0 ↦ ν7, 𝛼1 ↦ ν11 ⟧
        ",
        x
    )
    .parse()
    .unwrap();
    emu.dataize().0
}

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let input = args[1].parse().unwrap();
    let cycles = args[2].parse().unwrap();
    let mut total = 0;
    let mut f = 0;
    for _ in 0..cycles {
        f = fibo(input);
        total += f;
    }
    println!("{}-th Fibonacci number is {}", input, f);
    println!("Total is {}", total);
}

#[cfg(test)]
use simple_logger::SimpleLogger;

#[test]
fn calculates_fibonacci() {
    SimpleLogger::new().init().unwrap();
    assert_eq!(21, fibo(7))
}
