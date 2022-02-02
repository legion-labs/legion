# Pong example

## Launching

```sh
cargo mrun --bin pong -- --standalone
```

## Editing data

```sh
cargo mrun --bin editor-srv -- --project=examples/pong/data
cargo mrun --bin editor-client
```

## Data regeneration

```sh
cargo mrun --example pong_rebuild_data
```
