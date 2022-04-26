# Rust Guidelines

We are fortunate that Rust comes with a set of tools allowing us enforce a lot of good practices and guidelines automatically.
The tools we are relying on are:

- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny): Dependency linting, contains checks for licenses, [OVR](#rs-org-004---one-version-rule-ovr-enforcement) enforcement.
- [rustfmt](https://github.com/rust-lang/rustfmt): A tool for formatting Rust code according to style guidelines.
- [clippy](https://github.com/rust-lang/rust-clippy): A collection of lints to catch common mistakes and improve your Rust code.

The additional guidelines are used in places where we can't enforce them through these tools or when exceptions are necessary.

## **- RUST-001** - API Guidelines

We follow the base guidelines defined here: https://rust-lang.github.io/api-guidelines/.
If quoted in a review, specify the sections of interest.

## **- RUST-002** - Unsafe Code Guidelines

We follow the base guidelines defined here: https://rust-lang.github.io/unsafe-code-guidelines/.
If quoted in a review, specify the sections of interest.

## **- RUST-003** - binding/sys should live in their own repo

Bindings and sys crate should be designed to not rely on any of the legion crates and are considered leafs that we depend on. They will also not necessarily abide by the same guidelines we have for our monorepo.

## **- RUST-004** - Rustfmt defaults dictates formatting

At Legion Labs we rely purely on Rustfmt defaults. It allows us to have consistent formatting across our files and the rest of the rust ecosystem. As formatting can cause a lot of debate, we consider the debate closed in the context of rust.

To simplify your life consider enabling enabling `format on save` in your favorite editor.
For vscode, add the following to your settings:

```json
{
  "editor.formatOnSave": true
}
```

## **- RUST-005** - Only use permissive licenses

As part of our code is closed source and as we are licensing the open source part under permissive licensing, it is important to only rely on permissive licenses as well.
This is enforced in the deny.toml file at the root of each workspace of a repo. You can add licenses as we add dependencies but only permissive ones. In doubt on what constitutes a permissive license consider sending an email to staff@legionlabs.com or posting a question on our discord channel. We can involve our lawyers if need be.

Interesting links:

- https://tldrlegal.com/licenses/tags/Permissive
- https://spdx.org/licenses/

## **- RUST-006** - Enforce the One Version Rule (OVR)

Cargo brings easy and seamless dependency management to Rust, this can cause the proliferation of unmaintained dependencies and can cause maintainable issues when a given repo scales but also impacts executable size and performance in the long run. To help maintain our dependencies in check, we use cargo deny which lints our dependency usage. One of the practice of maintaining a healthy dependency usage is to rely on the one version rule, it means a dependency, even transitive can only exist in one version at any given time. It can be argued that cargo's use of duplicated dependency name mangling solves dependency hell management. But it doesn't come without any drawbacks, if two dependant expose a dependency type, it becomes incompatible between the different version. Any static variables or global state will be duplicated for each instance of a library.

For pragmatic reasons we allow exception to the rules (transitions between dependent versions in progress, production needs, etc...) you'll need to create an issue and schedule it to follow up on the duplicate use.

Create an issue next to the cargo deny exception in the skip section for each of the high level dependencies brining collisions:

```toml
skip = [
  # Intoduced by aws sdk [#1518](https://github.com/legion-labs/legion/issues/1518)
  { name = "hyper-rustls", version = "=0.22.1" },
  { name = "rustls-native-certs", version = "=0.5.0" },
  { name = "tokio-rustls", version = "=0.22.0" },
  { name = "tokio-util", version = "=0.6.9" },
```

## **- RUST-007** - Crate names should be in kebab case

To adhere with the conventions followed by most popular crates in the rust ecosystem, we choose to have crate names in kebab case. For more information see: https://github.com/rust-lang/api-guidelines/discussions/29#discussioncomment-233422.

## **- RUST-008** - Always prefer using an existing transitive dependency instead of bringing a new one

A dependency of a dependency is already used in the code base like it or not. If it's missing functionality we can contribute to it. If the dependent made a wrong choice we can try switching it upstream.
