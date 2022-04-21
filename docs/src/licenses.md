Legion Labs much like the Rust Project uses an Apache-2.0 / MIT dual license. The Apache license includes important protection against patent aggression, but it is not compatible with the GPL, version 2. To avoid problems using our code with GPLv2, it is alternately MIT licensed.

## Why?

The Apache License 2.0 makes sure that the user does not have to worry about infringing any patents by using the software. The user is granted a license to any patent that covers the software. This license is terminated if the user sues anyone over patent infringement related to this software. This condition is added in order to prevent patent litigations. However, the Apache license is incompatible with GPLv2. This is why Rust is dual-licensed as MIT/Apache (the "primary" license being Apache, MIT only for GPLv2 compat), and doing so would be wise for Legion Labs open source code. The MIT license requires reproducing countless copies of the same copyright header with different names in the copyright field, for every MIT library in use. The Apache license does not have this drawback. However, this is not the primary motivation for using the dual licensing.

## How?

Add the following to any open source repo README:

```md
### License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
```

Be sure to add the relevant LICENSE-{MIT,APACHE} files. You can copy these from the [Rust repo](https://github.com/rust-lang/rust) for a plain-text version.

And don't forget to update the license metadata in your Cargo.toml to:

```toml
license = "MIT OR Apache-2.0"
```

If the repo uses license headers, add the following boilerplate:

```rust
// Copyright 2022 Legion Labs Inc.
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
```
