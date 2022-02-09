# Pong example

## Launching

```sh
cargo mrun --bin pong -- --standalone
```

## Editing data

```sh
cargo mrun --bin editor-srv -- --project=examples/pong/data --scene "/scene.ent"
cargo mrun --bin editor-client
```

## Data regeneration

```sh
cargo mrun --example pong_rebuild_data
```

## Data exploration

```sh
cargo mrun --bin data-scrape -- configure --project examples/data --buildindex examples/data/temp
cargo mrun --bin data-scrape -- asset examples/pong/data/temp
```
