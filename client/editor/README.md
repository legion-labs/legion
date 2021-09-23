Editor client
=============

This folder contains the complete code for the Editor client.

The Editor client is a native web-app application based on
[Tauri](https://tauri.studio/en/) for its native side and on
[Vue.js](https://vuejs.org/) for its web-app side.

Because of its hybrid nature, building the Editor client requires a bit more
steps than running the usual `cargo build` command.

Structure of the application
----------------------------

The top folder (where this `README.md` currently sits) contains the usual Rust
binary crate with the main entrypoint. Additionally, this folder also contains a
`tauri.conf.json` file which instructs Tauri about some required configurations,
including where to find the web-app generated files or development server.

The Tauri application is currently configured to look for its web-app files in
the `frontend/dist` directory. By default, this directory does not exist and is
the result of building the web-app. It's absence will cause compilation (and
thus, `rust-analyzer` too) to fail.

As an effort to ease the development process, the [build script](./src/build.rs)
contains instructions to build the web-app automatically before building the
Rust native application. This is done through the execution of the `yarn build`
command inside the [`frontend`](./frontend) folder.

You may of course decide to run this step manually if you so chose, but building
the Rust native application will always call that step anyway.

*Breaking the build of the web-app will thus cause a compilation failure of the
*Rust native app. This is intended.**

Development mode
----------------

The default build process - while suitable for most situations - can prove
frustrating during development of the web-app: it is customary for
web-development to support a live/hot-reload flow to quickly iterate on UI
changes without having to rebuild everything.

Tauri has first-class support for that use case, and the application can be
built in `dev` mode, which instead of relying upon static web files, loads its
web content from a local development server.

To start the web-app development server, run the following command inside the
[`frontend`](./frontend) folder:

```bash
yarn serve
```

This will start a HTTP server listening on the `localhost` interface, on port
`8080`.

Once the development server is running, any change to the Vue.js application
will trigger an automatic reload without having to restart it.

To instruct the native application to use the development server instead of
static files, run the following command:

```bash
cargo run --no-default-features
```

This will disable the default `custom-protocol` feature that is used by Tauri to
conditionally compile the development server support in the application in place
of the static loading. You could of course also run `cargo build
--no-default-features` to generate a binary with identical abilities.

Note that when specifying `--no-default-features`, the `yarn build` process will
**not** be invoked at all, as its results would not be used anyway.

Divergence from the classic Tauri ways
--------------------------------------

Tauri was not exactly designed for our current directory layout. In most (all?)
examples out there, Tauri is an add-on over an existing web-app (be it Vue.js,
React.js or Angular.js).

Since our current approach is always *Rust first*, we sadly cannot rely on some
tools provided by the Tauri team, that are typically used to streamline certain
operations.

Notably, the `cargo tauri dev` and `cargo tauri build` commands are - for the
moment - not compatible and thus completely not supported.

We logged [an issue](https://github.com/tauri-apps/tauri/issues/2643) with them
to have these limitations lifted in the future.