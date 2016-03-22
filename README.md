# rb

[![Build Status](https://travis-ci.com/klingtnet/rb.svg?token=drwE1YPs35oqracubtuf&branch=master)](https://travis-ci.com/klingtnet/rb) [![license](https://img.shields.io/badge/license-GPL-blue.svg)](https://github.com/klingtnet/rb/blob/master/LICENSE) [![Crates.io](https://img.shields.io/crates/v/rustc-serialize.svg)](https://crates.io/crates/rb) [![rustdoc](https://img.shields.io/badge/rustdoc-hosted-blue.svg)](https://docs.klingt.net/rustdoc/rb)


A thread-safe fixed size circular (ring) buffer written in safe Rust.

## Features

- thread-safe
- blocking and non-blocking IO
- no unsafe blocks
- never under- or overflows

## Examples

```sh
cargo run --example saw
```

## Benchmark

The benchmarking feature needs *rust nightly*.

```sh
multirust run nightly -- cargo bench
```

It takes on my `Intel(R) Core(TM) i5-2520M CPU @ 2.50GHz` about `16ms` to push 2.8 million samples through the buffer in blocking IO mode.
