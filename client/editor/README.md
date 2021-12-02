# Editor client

This folder contains the complete code for the Editor client.

The Editor client is a native web-app application based on
[Tauri](https://tauri.studio/en/) for its native side and on
[Svelte](https://svelte.dev/) for its web-app side.

Because of its hybrid nature, building the Editor client requires a bit more
steps than running the usual `cargo build` command.

## Lauching the application

To simply build and launch the editor client, you can just type:

```
cargo run
```

This will build the Svelte frontend and embed it in the Rust binary before
running it. Simple.

### Development mode - Browser only

Since the application runs properly in browser you can skip the Tauri part.
That being said, keep in mind that the Tauri application is our priority
so make sure you've run and tested the application using `cargo run` before
pushing your code!

Go to the frontend folder:

```
cd frontend
```

And run the application:

```
yarn dev
```

Go to [http://localhost:3000](http://localhost:3000).

Once the feature you're working on is implemented, make sure you go back to
the client folder:

```
cd ..
```

And run the Tauri application:

```
cargo run
```

### Development mode - Tauri

You can also build and launch the editor client in development mode,
but things are a little bit more complex and you'll need
to run two processes **at the same time** (i.e. in 2 different terminals).

First the native Rust binary:

```
cargo run --no-default-features
```

Then the frontend:

```
cd frontend
yarn dev
```

## Structure of the application

The top folder (where this `README.md` currently sits) contains the usual Rust
binary crate with the main entrypoint. Additionally, this folder also contains a
`tauri.conf.json` file which instructs Tauri about some required configurations,
including where to find the web-app generated files or development server.

The Tauri application is currently configured to look for its web-app files in
the `frontend/dist` directory. By default, this directory in empty and will
be populated by the `yarn build` command that's ran in the `build.rs` file.

As an effort to ease the development process, the [build script](./src/build.rs)
contains instructions to build the web-app automatically before building the
Rust native application. This is done through the execution of the `yarn build` command inside the [`frontend`](./frontend) folder.

You may of course decide to run this step manually if you so chose, but building
the Rust native application will always call that step anyway.

*Breaking the build of the web-app will thus cause a compilation failure of the
*Rust native app. This is intended.\*\*

## Divergence from the classic Tauri ways

Tauri was not exactly designed for our current directory layout. In most (all?)
examples out there, Tauri is an add-on over an existing web-app (be it Svelte,
Vue, or React).

Since our current approach is always _Rust first_, we sadly cannot rely on some
tools provided by the Tauri team, that are typically used to streamline certain
operations.

Notably, the `cargo tauri dev` and `cargo tauri build` commands are - for the
moment - not compatible and thus completely not supported.

We logged [an issue](https://github.com/tauri-apps/tauri/issues/2643) with them
to have these limitations lifted in the future.

## Perf report

```
cargo run --bin perf-report
```
