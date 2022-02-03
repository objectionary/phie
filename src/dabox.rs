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
use std::fmt;

pub struct Dabox {
    pub object: isize,
    pub xi: usize,
    pub ret: Data,
    pub kids: [usize; 4],
}

impl Dabox {
    pub fn empty() -> Dabox {
        Dabox {
            object: -1,
            xi: 0,
            ret: 0,
            kids: [0; 4],
        }
    }

    pub fn start(ob: usize, xi: usize) -> Dabox {
        Dabox {
            object: ob as isize,
            xi,
            ret: 0,
            kids: [0; 4],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.object < 0
    }
}

impl fmt::Display for Dabox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "ν{}, ξ:#{}, r:0x{:04X}, {}",
            self.object, self.xi, self.ret,
            self.kids.to_vec().iter().map(|k| k.to_string()).collect::<Vec<String>>().join(".")
        )
    }
}

#[test]
fn makes_simple_dabox() {
    let mut dabox = Dabox::start(0, 0);
    dabox.ret = 42;
    assert_eq!(dabox.ret, 42);
}
