# Pong example

## Launching

```sh
cargo mrun --bin pong -- --standalone
```

## Editing data

```sh
cargo m run --bin editor-srv -- --project=examples/pong/data --scene "/scene.ent"
cargo m run --bin editor-client
```

## Data regeneration

```sh
cargo m run --bin pong_rebuild_data
```

## Data exploration

```sh
cargo m run --bin data-scrape -- configure --project examples/pong/data --buildindex examples/pong/data/temp
cargo m run --bin data-scrape -- asset examples/pong/data/temp
```
