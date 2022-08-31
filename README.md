<img alt="logo" src="https://www.objectionary.com/cactus.svg" height="100px" />

[![EO principles respected here](https://www.elegantobjects.org/badge.svg)](https://www.elegantobjects.org)
[![We recommend IntelliJ IDEA](https://www.elegantobjects.org/intellij-idea.svg)](https://www.jetbrains.com/idea/)

[![cargo](https://github.com/objectionary/phie/actions/workflows/cargo.yml/badge.svg)](https://github.com/objectionary/phie/actions/workflows/cargo.yml)
[![crates.io](https://img.shields.io/crates/v/phie.svg)](https://crates.io/crates/phie)
[![PDD status](http://www.0pdd.com/svg?name=objectionary/phie)](http://www.0pdd.com/p?name=objectionary/phie)
[![Hits-of-Code](https://hitsofcode.com/github/objectionary/phie)](https://hitsofcode.com/view/github/objectionary/phie)
![Lines of code](https://img.shields.io/tokei/lines/github/objectionary/phie)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/objectionary/phie/blob/master/LICENSE.txt)

It's an experimental emulator of a processor that understands
[𝜑-calculus](https://arxiv.org/abs/2111.13384) expressions, 
which is the formalism behind [EO](https://www.eolang.org) programming language.

To build it, install [Rust](https://www.rust-lang.org/tools/install) and then:

```bash
$ cargo build --release
```

If everything goes well, an executable binary will be in `target/release/fibonacci`:

```bash
$ target/release/fibonacci 7 40
```

This will calculate the 7th Fibonacci number 40 times.
Don't try to play with much larger numbers, this binary code is very slow. It's just an experiment.

To compile your own program instead of this primitive recursive Fibonacci calculator, you have to 
convert EO code into 𝜑-calculus expressions and then pass them to `Emu` struct like this:

```rust
use phie::emu::Emu;
pub fn main() {
    let emu: Emu = "
        ν0 ↦ ⟦ 𝜑 ↦ ν3 ⟧
        ν1 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2 ↦ ⟦ λ ↦ int-add, ρ ↦ ξ.𝛼0, 𝛼0 ↦ ξ.𝛼1 ⟧
        ν3 ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼0 ↦ ν1, 𝛼1 ↦ ν1 ⟧
        ν5 ↦ ⟦ 𝜑 ↦ ν3(ξ) ⟧
    ".parse().unwrap();
    let dtz = emu.dataize();
    print!("The result is: {}", dtz.0);
}
```

This code is equivalent to the following EO code:

```text
[] > foo
  42 > x
  x.add x > @
```

But in a more "functional" way:

```text
[] > foo
  42 > x
  int-add > @
    x
    x
```

More tests are in `src/emu.rs` file.
