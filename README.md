# Env parser

[![Latest Version](https://img.shields.io/crates/v/env_parser.svg)](https://crates.io/crates/env_parser)
[![Build Status](https://img.shields.io/github/workflow/status/jasperav/env_parser/CI/master)](https://github.com/jasperav/env_parser/actions)

This crate will parse an `.env` file. By implementing the `Transformer` trait, you can 
customize the behaviour after processing key value pair in the env file. Comments are preserved and 
can be used on top of the Rust mapped property.

For convenience there is a feature `to_lazy_static` that will automatically map key value pairs into a `lazy_static` block.

Check `assert_test.rs` and `assert_test_lazy.rs` for examples.

## Usage

Add the following to your Cargo.toml...

```toml
[dependencies]
env_parser = "*"
```

Ideally, create `build.rs` file and call the `env_parser` reader from the build file, so that your mapped Rust file
is always in sync with your `.env` file.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
