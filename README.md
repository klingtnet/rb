# rb

[![Build Status](https://travis-ci.org/klingtnet/rb.svg?branch=master)](https://travis-ci.org/klingtnet/rb)
[![Build Status (appveyor)](https://ci.appveyor.com/api/projects/status/ixq6ai1c96ggm4fr?svg=true)](https://ci.appveyor.com/project/klingtnet/rb)
![license](https://img.shields.io/badge/license-MIT%2FApache%202.0-blue.svg)
[![rustdoc](https://img.shields.io/badge/rustdoc-hosted-blue.svg)](https://docs.rs/rb)
[![dependency status](https://deps.rs/repo/github/klingtnet/rb/status.svg)](https://deps.rs/repo/github/klingtnet/rb)

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
rustup run nightly -- cargo bench
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
