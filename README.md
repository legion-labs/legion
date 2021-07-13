[![CI](https://github.com/legion-labs/legion/actions/workflows/ci.yml/badge.svg)](https://github.com/legion-labs/legion/actions/workflows/ci.yml)

# Legion Monorepo

This is the  mono repository of legion, it contains all the application code of the engine itself, the tools and the pipeline.

Visit https://legion-labs.github.io/legion for the full documentation.

## Setting up your environment:

### Build time dependencies:

Legion depends on the following for building:

* cmake
* ninja
* python3
* Cuda
* Vulkan SDK

On windows and using `scoop`:

```powershell
scoop install cmake ninja python
scoop install legion-labs/cuda legion-labs/vulkan
```

### License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
