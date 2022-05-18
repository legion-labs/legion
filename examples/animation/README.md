# Animation example

## Launching

```sh
cargo m run --bin runtime-srv --features=standalone -- --manifest=examples/animation/data/runtime/game.manifest --root-asset="(1d9ddd99aad89045,1fa058cb-5877-5ffe-dcb7-1f364a804a8f)"
```

## Editing data

```sh
cargo m run --bin editor-srv -- --project-root=./target/data/workspaces/animation --repository-name=examples-animation --manifest=examples/animation/data/runtime/game.manifest --scene "/scene.ent" --build-output-database-address=./target/output_db
cargo m run --bin editor-client
```

## Data regeneration

```sh
cargo m run --bin animation-rebuild-data
```

## Data exploration

```sh
cargo m run --bin data-scrape -- configure --project examples/animation/data --output temp/
cargo m run --bin data-scrape -- asset examples/animation/data/temp
```