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

use eoc::atom::*;
use eoc::data::Data;
use eoc::emu::{Emu, ROOT_BX};
use eoc::object::Object;
use eoc::path::{Item, Path};
use eoc::ph;
use std::env;
use std::str::FromStr;
use simple_logger::SimpleLogger;

pub fn fibo(x: Data) -> Result<Data, String> {
    let mut emu = Emu::empty();
    emu.put(1, Object::dataic(x));
    emu.put(
        2,
        Object::open()
            .with(Item::Phi, ph!("v3"), true)
            .with(Item::Attr(0), ph!("v1"), false),
    );
    emu.put(3, Object::open().with(Item::Phi, ph!("v13"), false));
    emu.put(5, Object::dataic(2));
    emu.put(
        6,
        Object::atomic(int_sub)
            .with(Item::Rho, ph!("$.0"), false)
            .with(Item::Attr(0), ph!("v5"), false),
    );
    emu.put(7, Object::dataic(1));
    emu.put(
        8,
        Object::atomic(int_sub)
            .with(Item::Rho, ph!("$.0"), false)
            .with(Item::Attr(0), ph!("v7"), false),
    );
    emu.put(
        9,
        Object::open()
            .with(Item::Phi, ph!("v3"), true)
            .with(Item::Attr(0), ph!("v8"), false),
    );
    emu.put(
        10,
        Object::open()
            .with(Item::Phi, ph!("v3"), true)
            .with(Item::Attr(0), ph!("v6"), false),
    );
    emu.put(
        11,
        Object::atomic(int_add)
            .with(Item::Rho, ph!("v9"), false)
            .with(Item::Attr(0), ph!("v10"), false),
    );
    emu.put(
        12,
        Object::atomic(int_less)
            .with(Item::Rho, ph!("$.0"), false)
            .with(Item::Attr(0), ph!("v5"), false),
    );
    emu.put(
        13,
        Object::atomic(bool_if)
            .with(Item::Rho, ph!("v12"), false)
            .with(Item::Attr(0), ph!("v7"), false)
            .with(Item::Attr(1), ph!("v11"), false),
    );
    let bx = emu.new(2, ROOT_BX);
    emu.log();
    let f = emu.dataize(bx)?;
    emu.delete(bx);
    Ok(f)
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let input = args[1].parse().unwrap();
    let cycles = args[2].parse().unwrap();
    let mut total = 0;
    let mut f = 0;
    for _ in 0..cycles {
        f = fibo(input).unwrap();
        total += f;
    }
    println!("{}-th Fibonacci number is {}", input, f);
    println!("Total is {}", total);
}

#[test]
fn calculates_fibonacci() {
    SimpleLogger::new().init().unwrap();
    assert_eq!(87, fibo(3).unwrap())
}

