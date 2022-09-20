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
        ν0(𝜋) ↦ ⟦ 𝜑 ↦ ν3(𝜋) ⟧
        ν1(𝜋) ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ 𝜋.𝛼0, 𝛼0 ↦ 𝜋.𝛼1 ⟧
        ν3(𝜋) ↦ ⟦ 𝜑 ↦ ν2(ξ), 𝛼0 ↦ ν1(𝜋), 𝛼1 ↦ ν1(𝜋) ⟧
        ν5(𝜋) ↦ ⟦ 𝜑 ↦ ν3(ξ) ⟧
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

Run and fix [Clippy](https://github.com/rust-lang/rust-clippy) lints issues before committing changes:

1. Install [Rustup](https://rustup.rs/). If Rustup was already installed, update to ensure have the latest Rustup and compiler.

```bash
$ rustup update
```

2. Install Clippy.

```bash
$ rustup component add clippy
```

3. Run Clippy.

```bash
$ cargo clippy
```

4. Automatically applying Clippy suggestions (Not all issues will be fixed automatically. 
Also, Clippy has some bugs with false-positive cases for some lints, so better to check automaticall fixes as well).

```bash
$ cargo clippy --fix
```
