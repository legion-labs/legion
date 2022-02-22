# Physics example

## Launching

```sh
cargo m run --bin runtime-srv -- --standalone
```

## Editing data

```sh
cargo m run --bin editor-srv -- --project=examples/physics/data --scene "/scene.ent"
cargo m run --bin editor-client
```

## Data regeneration

```sh
cargo m run --bin physics_rebuild_data
```

## Data exploration

```sh
cargo m run --bin data-scrape -- configure --project examples/data --buildindex examples/data/temp
cargo m run --bin data-scrape -- asset examples/pong/data/temp
```
