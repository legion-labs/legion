# The **Legion** Interactive Content Creation Platform

Legion is ... . Visit [here](https://legionengine.com) for more information.

Legion Labs welcomes [contributions] from everyone, please read [this](./CONTRIBUTING.md) for more information.

---

<p align="center">
   <img alt="Legion Labs" src="https://legionlabs.com/images/logo.png" width="340" >
</p>

---

<p align="center">
   <img alt="Legion Engine" src="https://github.com/legion-labs/legion/raw/main/build/snapshot.png" style="max-width: 100%;">
</p>

---

<p align="center">
    <a href="https://github.com/legion-labs/legion/actions/workflows/ci_test.yml"><img src="https://github.com/legion-labs/legion/actions/workflows/ci_test.yml/badge.svg" alt="CI - Test" style="max-width: 100%;"></a>
    <a href="https://github.com/legion-labs/legion/actions/workflows/release.yml"><img src="https://github.com/legion-labs/legion/actions/workflows/release.yml/badge.svg" alt="CI - Release" style="max-width: 100%;"></a>
    <a href="https://cov.legionengine.com/index.html" rel="nofollow"><img src="https://github.com/legion-labs/legion/raw/main/build/coverage.svg" alt="Coverage"  style="max-width: 100%;"></a>
    <a href="https://www.rust-lang.org/tools/install"><img src="https://img.shields.io/badge/msrv-1.57-green" alt="MSRV" style="max-width: 100%;"></a></p>
</p>

---

## Getting Started

The repo contains all the application code of the engine itself, the tools and the pipeline.

- Visit https://book.legionengine.com for the legion engine book.
- Visit https://api.legionengine.com for the legion api reference book.

We currently don't support pre-built packages, but you can build and run locally all the components necessary to build the engine and it's tools.

### Dev Environment Setup

#### Windows setup

First you need to have a valid Visual Studio 2019 or above toolchain installed, if you don't you can install the [Visual Studio build tools instead](https://aka.ms/vs/17/release/vs_BuildTools.exe) with C++ based development packages. For the remaining dependencies we recommend using [scoop](https://scoop.sh/) to install the following:

- Rust dependencies by running the following commands on a powershell prompt:

```powershell
scoop install rustup-msvc
scoop install cmake
scoop install ninja
scoop install nasm
```

- Front end dependencies by running the following commands on a powershell prompt:

```powershell
scoop install nvm
scoop install protobuf
nvm install 16.10.0
nvm use 16.10.0
npm -g i pnpm
```

On two instances of a powershell prompt and at the root of this repo run the following:

```powershell
cargo mrun --p editor-srv
```

On the second terminal:

```powershell
cargo mrun --p editor-client
```

#### Linux setup

Linux steps here.

## Community

Info here ... .

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
