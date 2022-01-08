[![CI - Test](https://github.com/legion-labs/legion/actions/workflows/ci_test.yml/badge.svg)](https://github.com/legion-labs/legion/actions/workflows/ci_test.yml)
[![CI - Release](https://github.com/legion-labs/legion/actions/workflows/release.yml/badge.svg)](https://github.com/legion-labs/legion/actions/workflows/release.yml)
[![Coverage](https://cov.legionengine.com/badges/flat.svg)](https://cov.legionengine.com/index.html)
![MSRV](https://img.shields.io/badge/msrv-1.57-green)

# Legion Monorepo

This is the mono repository of legion, it contains all the application code of the engine itself, the tools and the pipeline.

- Visit https://book.legionengine.com for the legion engine book.
- Visit https://api.legionengine.com for the legion api reference book.
- Visit https://cov.legionengine.com for the legion code coverage statistics.
- Visit https://build-timings.legionengine.com for build time statistic.

## Setting up your environment:

### Build time dependencies:

Legion depends on the following for building:

- cmake
- ninja
- python3
- Vulkan SDK

On windows we recommand using scoop using `scoop`:

```powershell
scoop install cmake ninja python
scoop install legion-labs/vulkan
```

### License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
