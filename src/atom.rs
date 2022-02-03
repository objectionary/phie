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

use crate::data::Data;
use crate::emu::Emu;
use crate::path::Item;

pub type Atom = fn(&mut Emu, usize) -> Data;

pub fn int_add(emu: &mut Emu, bx: usize) -> Data {
    emu.calc(bx, Item::Rho) + emu.calc(bx, Item::Attr(0))
}

pub fn int_sub(emu: &mut Emu, bx: usize) -> Data {
    emu.calc(bx, Item::Rho) - emu.calc(bx, Item::Attr(0))
}

pub fn int_less(emu: &mut Emu, bx: usize) -> Data {
    (emu.calc(bx, Item::Rho) < emu.calc(bx, Item::Attr(0))) as Data
}

pub fn bool_if(emu: &mut Emu, bx: usize) -> Data {
    if emu.calc(bx, Item::Rho) == 1 {
        emu.calc(bx, Item::Attr(0))
    } else {
        emu.calc(bx, Item::Attr(1))
    }
}
