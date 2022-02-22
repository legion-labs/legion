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
cargo mrun --bin pong_rebuild_data
```

## Data exploration

```sh
cargo mrun --bin data-scrape -- configure --project examples/pong/data --buildindex examples/pong/data/temp
cargo mrun --bin data-scrape -- asset examples/pong/data/temp
```
