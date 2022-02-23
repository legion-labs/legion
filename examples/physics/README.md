# Physics example

## Launching

```sh
cargo m run --bin runtime-srv -- --project=examples/physics/data --standalone --physics-debugger
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
cargo m run --bin data-scrape -- configure --project examples/physics/data --buildindex examples/physics/data/temp
cargo m run --bin data-scrape -- asset examples/physics/data/temp
```
