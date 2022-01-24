[![make](https://github.com/yegor256/phi-emu/actions/workflows/cargo.yml/badge.svg)](https://github.com/yegor256/phi-emu/actions/workflows/cargo.yml)

It's an experimental emulator of 𝜑PU. It can turn a simple 
[EO](https://www.eolang.org) program into 𝜑ASM binary structure
and then execute it. 

To build it, install [Rust](https://www.rust-lang.org/tools/install) and then:

```bash
$ cargo build --release
```

If everything goes well, an executable binary will be in `target/release/fibonacci`:

```bash
$ target/release/fibonacci 17 1000
```

This will calculate the 17th Fibonacci number 1000 times.
