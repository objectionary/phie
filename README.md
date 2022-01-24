[![make](https://github.com/yegor256/phi-emu/actions/workflows/make.yml/badge.svg)](https://github.com/yegor256/phi-emu/actions/workflows/make.yml)

It's a simple emulator of phiPU that can execute 
[EO](https://www.eolang.org) programs. 

To build it, just make sure your code is in `program.h` and then run:

```bash
$ cargo build --release
```

If everything goes well, an executable binary will be in `target/release/fibonacci`:

```bash
$ time target/release/fibonacci 17 1000
```

This will calculate 17th Fibonacci number 1000 times.

You need to have [Rust](https://www.rust-lang.org/tools/install) installed.
