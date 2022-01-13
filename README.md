<h1 align="center">
    <br/>
    <a href="https://legionengine.com"><img alt="Legion Engine" src="https://github.com/legion-labs/legion/raw/main/.github/images/legion-engine-logo.svg", width="340px" ></a>
    <br/>
    <br/>
</h1>

<p align="center">
    <b>Legion</b> is an interactive content creation <b>pipeline</b> that focusses on bringing <b>fast iteration</b> times at the scale of a <b>production</b> by providing collaborative tooling and automation.
    <br/>
    <br/>
    From the <b>inception</b> of an idea, to having it in the hands of the <b>players</b>, Legion's goal is to allow you to focus on <b>finding the fun</b>.
</p>

<p align="center">
    <a href="https://github.com/legion-labs/legion/actions/workflows/ci_test.yml"><img src="https://github.com/legion-labs/legion/actions/workflows/ci_test.yml/badge.svg" alt="CI - Test" style="max-width: 100%;"></a>
    <a href="https://github.com/legion-labs/legion/actions/workflows/release.yml"><img src="https://github.com/legion-labs/legion/actions/workflows/release.yml/badge.svg" alt="CI - Release" style="max-width: 100%;"></a>
    <a href="https://cov.legionengine.com/index.html" rel="nofollow"><img src="https://github.com/legion-labs/legion/raw/main/.github/images/coverage.svg" alt="Coverage"  style="max-width: 100%;"></a>
    <a href="https://www.rust-lang.org/tools/install"><img src="https://img.shields.io/badge/msrv-1.57-green" alt="MSRV" style="max-width: 100%;"></a></p>
</p>

<p align="center">
   <img alt="Legion Engine" src="https://github.com/legion-labs/legion/raw/main/.github/images/snapshot.png" style="max-width: 100%;">
</p>

---

Legion Engine is made of multiple components:

- Cloud native data processing pipeline, where the size and management artifact is a thing of the past.
- Source control solution capable of handling large binary files while providing merge guarantees through locking, even across branches.
- Fully integrated and scalable telemetry solution, allowing to have detailed visibility on all aspects of your builds, from the production floor to the live environment.
- Vulkan based streaming solution, allowing the editor to operate in hybrid mode and keeping the heavy lifting on the backend.
- Web technologies powered editor, providing an accessible but yet powerful experience to everyone, maybe even players.
- Scripting and hot reloading capable engine runtime, bridging a fast iteration loop to everyone on the production.

Legion Engine is an open source engine to the limit of what Legion Labs has control over. We firmly believe that the next game development technology stack is going to be an open source one, and as such Legion Labs has a commitment to nurture and grow a community of developers that are able to contribute to the engine and make it theirs as well!

## ⚡️ Quick Start

The repo contains all the application code of the engine itself, the tools and the pipeline. A complete guide to getting setup and getting to work on the engine is described [here](https://book.legionengine.com), for an overview of all the libraries used can be found [here](https://api.legionengine.com).

We currently don't support pre-built packages, but you can build and run locally all the components necessary to build the engine and it's tools. For a quick setup follow the guides below, We support Windows and Linux as our main development platforms and targets. The plan is to support MacOs as well.

> This being a monorepo, it relies on some extra tooling to work around some Cargo limitations around monorepo and package selection. You will need to use the cargo command you are accustomed to like `cargo clippy`, `cargo run`, but you need to prefix them with `cargo mclippy`, `cargo mrun`. More information surrounding this is available [here]((https://book.legionengine.com/link_to_monorepo_tooling).

<details><summary><b>Windows setup instructions</b></summary>

First you need to have a valid Visual Studio 2019 or above toolchain installed, if you don't you can install the [Visual Studio build tools instead](https://aka.ms/vs/17/release/vs_BuildTools.exe) with `Desktop Development with C++` packages. For the remaining dependencies we recommend using [scoop](https://scoop.sh/) to install the necessary dependencies:

- on a powershell prompt (locate and select powershell on the Start menu)

```powershell
Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')
```

- if you get an error you might need to change the execution policy with, the repeat the previous step:

```
Set-ExecutionPolicy RemoteSigned -scope CurrentUser
```

- Add Legion Labs bucket and the extras bucket

```powershell
scoop bucket add legion-labs https://github.com/legion-labs/scoop-bucket
scoop bucket add extras
```

- install Rust dependencies by running the following commands on a powershell prompt:

```powershell
scoop install rustup-msvc
scoop install legion-labs/vulkan
scoop install cmake
scoop install ninja
scoop install nasm
```

- Install the front end dependencies by running the following commands on a powershell prompt:

```powershell
scoop install nvm
scoop install protobuf
nvm install 16.10.0
nvm use 16.10.0
npm -g i pnpm
```

</details>

<details><summary><b>Linux setup instructions</b></summary>

Linux steps here.

</details>

After finishing the setup, on two instances of your prompt, run the following commands at the root of the repo:

```
cargo mrun -p editor-srv
```

On the second terminal:

```
cargo mrun -p editor-client
```

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

---

<p align="center">
    <a href="https://legionlabs.com"><img alt="Legion Labs" src="https://github.com/legion-labs/legion/raw/main/.github/images/legion-labs-logo.svg", width="240px" ></a>
</p>
