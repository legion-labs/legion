# Pong example

## Launching

```sh
cargo run --bin pong -- --standalone
```

## Editing data

```sh
cargo run --bin editor-srv -- --project=examples/pong/data
cargo run --bin editor-client
```

## Manual data compilation

- delete `examples/pong/data/project.index`
- delete `examples/pong/data/temp`

```sh
cargo run --bin data-build create examples/pong/data/temp --project=..
cargo run --bin data-build compile "(1c0ff9e497b0740f,29b8b0d0-ee1e-4792-aca2-3b3a3ce63916)|1d9ddd99aad89045" --buildindex=examples/pong/data/temp --cas=examples/pong/data/temp --target=game --platform=windows --locale=en
```
