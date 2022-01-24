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

use phi_emu::directives::Directives;
use phi_emu::emu::Emu;
use phi_emu::obs::Obs;
use phi_emu::path::Path;
use phi_emu::ph;
use std::str::FromStr;

pub fn main() {
    let emu = Emu {
        obses: Vec::from([
            Obs::Data(ph!("v3"), 0x11),                             // v0
            Obs::Copy(ph!("v2"), vec![ph!("v0")]),                  // v1
            Obs::Abstract(ph!("v12"), vec![]),                      // v2
            Obs::Empty,                                             // v3
            Obs::Data(ph!("v3"), 0x02),                             // v4
            Obs::Atom(7, ph!("$.0"), vec![ph!("v4")]),              // v5
            Obs::Data(ph!("v3"), 0x01),                             // v6
            Obs::Atom(7, ph!("$.0"), vec![ph!("v6")]),              // v7
            Obs::Copy(ph!("v2"), vec![ph!("v7")]),                  // v8
            Obs::Copy(ph!("v2"), vec![ph!("v5")]),                  // v9
            Obs::Atom(11, ph!("v8"), vec![ph!("v9")]),              // v10
            Obs::Atom(5, ph!("$.0"), vec![ph!("v4")]),              // v11
            Obs::Atom(15, ph!("v11"), vec![ph!("v6"), ph!("v10")]), // v12
            Obs::Empty,                                             // v13
        ]),
        directives: Directives::parse(
            "
            int.less::
              DATAIZE $.^
              DATAIZE $.0
              SUB $.0 FROM $.^ TO r1
              JUMP less IF r1 GT
              WRITE 0x00 TO r2
              RETURN r2
              less:
              WRITE 0x01 TO r2
              RETURN r2
            int.sub::
              DATAIZE $.^
              DATAIZE $.0
              SUB $.0 FROM $.^ TO r1
              RETURN r1
            int.add::
              DATAIZE $.^
              DATAIZE $.0
              ADD $.^ AND $.0 TO r1
              RETURN r1
            bool.if::
              DATAIZE $.^
              SUB 0x01 FROM $.^ TO r1
              JUMP yes IF r1 EQ
              DATAIZE $.1
              READ $.1 TO r1
              RETURN r1
              yes:
              DATAIZE $.0
              READ $.0 TO r1
              RETURN r1
            ",
        ),
        ..Default::default()
    };
    let fibo = emu.dataize(0);
    print!("17th Fibonacci number is {}\n", fibo)
}
