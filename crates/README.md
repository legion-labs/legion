# Core libraries and Bevy basis

In August 2021 we decided to bootstrap the engine runtime to use bevy as a fork. As we are targeting AAA/AA developers the team felt like we need to take ownership of that part of the code as we are experimenting with ways to do that in Rust. We will try to contribute back to Bevy if it doesn't go against their direction.

> It is important to keep the Bevy Authors in the author list of the Bevy imported crates.

## Crate mapping

The following crates are heavily based on their Bevy counterparts:

| Legion                                                                                           | Bevy                                                                                             | Notes                      |
| ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ | -------------------------- |
| [crates/lgn-app](https://github.com/legion-labs/legion/tree/main/crates/lgn-app)                 | [crates/bevy_app](https://github.com/legion-labs/bevy/tree/main/crates/bevy_app)                 |                            |
| [crates/lgn-core](https://github.com/legion-labs/legion/tree/main/crates/lgn-core)               | [crates/bevy_core](https://github.com/legion-labs/bevy/tree/main/crates/bevy_core)               |                            |
| [crates/lgn-derive](https://github.com/legion-labs/legion/tree/main/crates/lgn-derive)           | [crates/bevy_derive](https://github.com/legion-labs/bevy/tree/main/crates/bevy_derive)           |                            |
| [crates/lgn-ecs](https://github.com/legion-labs/legion/tree/main/crates/lgn-ecs)                 | [crates/bevy_ecs](https://github.com/legion-labs/bevy/tree/main/crates/bevy_ecs)                 | include `macros` sub-crate |
| [crates/lgn-gilrs](https://github.com/legion-labs/legion/tree/main/crates/lgn-gilrs)             | [crates/bevy_gilrs](https://github.com/legion-labs/bevy/tree/main/crates/bevy_gilrs)             |                            |
| [crates/lgn-hierarchy](https://github.com/legion-labs/legion/tree/main/crates/lgn-hierarchy)     | [crates/bevy_hierarchy](https://github.com/legion-labs/bevy/tree/main/crates/bevy_hierarchy)     |                            |
| [crates/lgn-input](https://github.com/legion-labs/legion/tree/main/crates/lgn-input)             | [crates/bevy_input](https://github.com/legion-labs/bevy/tree/main/crates/bevy_input)             |                            |
| [crates/lgn-macro-utils](https://github.com/legion-labs/legion/tree/main/crates/lgn-macro-utils) | [crates/bevy_macro_utils](https://github.com/legion-labs/bevy/tree/main/crates/bevy_macro_utils) |                            |
| [crates/lgn-math](https://github.com/legion-labs/legion/tree/main/crates/lgn-math)               | [crates/bevy_math](https://github.com/legion-labs/bevy/tree/main/crates/bevy_math)               |                            |
| [crates/lgn-ptr](https://github.com/legion-labs/legion/tree/main/crates/lgn-ptr)                 | [crates/bevy_ptr](https://github.com/legion-labs/bevy/tree/main/crates/bevy_ptr)                 |                            |
| [crates/lgn-tasks](https://github.com/legion-labs/legion/tree/main/crates/lgn-tasks)             | [crates/bevy_tasks](https://github.com/legion-labs/bevy/tree/main/crates/bevy_tasks)             |                            |
| [crates/lgn-time](https://github.com/legion-labs/legion/tree/main/crates/lgn-time)               | [crates/bevy_time](https://github.com/legion-labs/bevy/tree/main/crates/bevy_time)               |                            |
| [crates/lgn-transform](https://github.com/legion-labs/legion/tree/main/crates/lgn-transform)     | [crates/bevy_transform](https://github.com/legion-labs/bevy/tree/main/crates/bevy_transform)     |                            |
| [crates/lgn-utils](https://github.com/legion-labs/legion/tree/main/crates/lgn-utils)\*           | [crates/bevy_utils](https://github.com/legion-labs/bevy/tree/main/crates/bevy_utils)             | labels.rs                  |
| [crates/lgn-window](https://github.com/legion-labs/legion/tree/main/crates/lgn-window)           | [crates/bevy_window](https://github.com/legion-labs/bevy/tree/main/crates/bevy_window)           |                            |
| [crates/lgn-winit](https://github.com/legion-labs/legion/tree/main/crates/lgn-winit)             | [crates/bevy_winit](https://github.com/legion-labs/bevy/tree/main/crates/bevy_winit)             |                            |

\*: only integrates a subset of source files

## Integration history

- [PR 709](https://github.com/legion-labs/legion/pull/709)
  - reference: [7356f15](https://github.com/bevyengine/bevy/commit/7356f1586d74039f840bcfcf24af3e21c23e3c18) (December 15th 2021)
- [PR 880](https://github.com/legion-labs/legion/pull/880)
  - reference: [cb2ba19](https://github.com/bevyengine/bevy/commit/cb2ba19d97ecb8f878c26357ade2ea7bcbd0cbc9) (January 17th 2022)
- [PR 1046](https://github.com/legion-labs/legion/pull/1046)
  - reference: [c4f132a](https://github.com/bevyengine/bevy/commit/c4f132afbfe5688afd13f9b05040dfdf98b65489) (February 17th 2022)
- [PR 1232](https://github.com/legion-labs/legion/pull/1232)
  - reference: [024d984](https://github.com/bevyengine/bevy/commit/024d98457c80d25f9d5269f214d31e9967cc734a) (March 22nd 2022)
- [PR 1596](https://github.com/legion-labs/legion/pull/1596)
  - reference: [328c26d](https://github.com/bevyengine/bevy/commit/328c26d02c50de0bc77f0d24a376f43ba89517b1) (April 27th 2022)
- [PR 2092](https://github.com/legion-labs/legion/pull/2092)
  - reference: [86dd6f0](https://github.com/bevyengine/bevy/commit/86dd6f065d8d355ca6d75ee3ea270b9dad7e8ecd) (June 21st 2022)
