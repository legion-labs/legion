# Physics engine - Research

## Open source physics engines

| Name | Repository | License | Language | Platforms | Notes |
|--|--|--|--|--|--|
| [Bullet Physics](https://pybullet.org/wordpress) | [GitHub](https://github.com/bulletphysics/bullet3) | [zlib](http://opensource.org/licenses/Zlib) | C++, Python wrapper | | |
| [Chrono](https://projectchrono.org/) | [GitHub](https://github.com/projectchrono/chrono) | [BSD-3](https://github.com/projectchrono/chrono/blob/develop/LICENSE) | C++,  Python wrapper | Windows, Linux, OS X | |
| [nphysics](https://nphysics.org/) | [GitHub](https://github.com/dimforge/nphysics) | Apache-2.0 | **Rust** | * | Passively maintained, superseded by Rapier. [Interactive demos](http://demo.nphysics.org) |
| [Open Dynamics Engine (ODE)](http://www.ode.org/) | [BitBucket](https://bitbucket.org/odedevs/ode/src/master/) | BSD or LGPL | C/C++ | ? | |
| [PhysX](https://developer.nvidia.com/physx-sdk) | [GitHub](https://github.com/NVIDIAGameWorks/PhysX) | BSD-3 | C++ | iOS, MacOS, Android ARM, Linux, Windows | [Developer guide](https://gameworksdocs.nvidia.com/simulation.html). Does not require NVIDIA GPU, but will take advantage of if present. In addition to BSD-3 licensed platforms, unchanged NVIDIA EULA platforms: X1, PS4, Switch. Integrated as built-in 3D physics engine for Unity, except when using DOTS (data-oriented) stack which uses a [proprietary engine](https://unity.com/unity/physics) that can be combined with Havok (additional reading: [Introduction to Unity physics](https://docs.unity3d.com/Packages/com.unity.physics@0.0/manual/index.html?_gl=1*oc05n9*_ga*NDc1ODQ2OTk3LjE2MjgxMDgzMzI.*_ga_1S78EFL1W5*MTYyODEwODM0Ni4xLjEuMTYyODEwODY4MS42MA..&_ga=2.236949249.1220063820.1628108332-475846997.1628108332)). Also integrated in Unreal (v3+4), and Stingray (Autodesk, discontinued?). |
| [physx](https://crates.io/crates/physx) | [GitHub](https://github.com/EmbarkStudios/physx-rs) | MIT or Apache-2.0 | Rust | * | [Embark Studios](https://www.embark-studios.com/) maintains unofficial Rust bindings for PhysX in this high-level crate, and also the unsafe low-level crate [physx-sys](https://crates.io/crates/physx-sys/0.4.14). |
| [Rapier](https://rapier.rs) | [GitHub](https://github.com/dimforge/rapier) | Apache-2.0 | **Rust** | * | Part of Dimforge. [Roadmap](https://www.dimforge.com/blog/2021/01/01/physics-simulation-with-rapier-2021-roadmap/#rapier-roadmap-for-2021) |
| [ReactPhysics3D](https://www.reactphysics3d.com/) | [GitHub](https://github.com/DanielChappuis/reactphysics3d) | Zlib | C++ | Windows, Linux, OS X | |
