# Physics example

## Launching

```sh
cargo m run --bin runtime-srv --features=standalone -- --manifest=examples/physics/data/runtime/game.manifest --root="(1d9ddd99aad89045,1fa058cb-5877-5ffe-dcb7-1f364a804a8f)" --physics-debugger
```

## Editing data

```sh
cargo m run --bin editor-srv -- --project-root=./target/data/workspaces/physics --repository-name=examples-physics --manifest=examples/physics/data/runtime/game.manifest --scene "/scene.ent" --build-output-database-address=./target/output_db
cargo m run --bin editor-client
```

## Data regeneration

```sh
cargo m run --bin physics-rebuild-data
```

## Data exploration

```sh
cargo m run --bin data-scrape -- configure --project examples/physics/data --output temp/
cargo m run --bin data-scrape -- asset examples/physics/data/temp
```
