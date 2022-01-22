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

use crate::primitives::*;

#[derive(Default, Clone)]
pub struct Obs {
    pub atom: i16,
    pub data: Option<Data>,
    pub phi: Path,
    pub rho: Path,
    pub sup: Path,
    pub args: Vec<Path>
}

impl Obs {
    pub fn with(self, a: Path) -> Obs {
        Obs {
            args: [self.args, vec![a]].concat(),
            ..self
        }
    }

    pub fn empty() -> Obs {
        Obs {
            ..Default::default()
        }
    }

    pub fn data(sup: Path, data: Data) -> Obs {
        Obs {
            sup,
            data: Some(data),
            ..Default::default()
        }
    }

    pub fn copy(sup: Path) -> Obs {
        Obs {
            sup,
            ..Default::default()
        }
    }

    pub fn atom(atom: i16, rho: Path) -> Obs {
        Obs {
            atom, rho,
            ..Default::default()
        }
    }

    pub fn decorate(phi: Path) -> Obs {
        Obs {
            phi,
            ..Default::default()
        }
    }
}

#[test]
pub fn makes_empty_obs() {
    assert!(Obs::empty().args.is_empty())
}
