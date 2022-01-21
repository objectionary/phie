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

extern crate phi_emu;

use phi_emu::Emu;
use phi_emu::obs::Obs;

pub fn main() {
    let emu = Emu {
        obses: Vec::from([
            Obs { sup: "v3", data: 0x11, ..Default::default() }, // v0
            Obs { sup: "v2", a0: "v0", ..Default::default() }, // v1
            Obs { phi: "v12", ..Default::default() }, // v2
            Obs::empty(), // v3
            Obs { sup: "v3", data: 0x02, ..Default::default() }, // v4
            Obs { atom: 7, rho: "$.0", a0: "v4", ..Default::default() }, // v5
            Obs { sup: "v3", data: 0x01, ..Default::default() }, // v6
            Obs { atom: 7, rho: "$.0", a0: "v6", ..Default::default() }, // v7
            Obs { sup: "v2", a0: "v7", ..Default::default() }, // v8
            Obs { sup: "v2", a0: "v5", ..Default::default() }, // v9
            Obs { atom: 11, rho: "v8", a0: "v9", ..Default::default() }, // v10
            Obs { atom: 5, rho: "$.0", a0: "v4", ..Default::default() }, // v11
            Obs { atom: 15, rho: "v11", a0: "v6", a1: "v10", ..Default::default() }, // v12
            Obs::empty(), // v13
        ]),
        ..Default::default()
    };
    print!("Hello, world!")
}
