# Pong example



## Data regeneration
Before playing, you will need to generate the data

```sh
cargo m run --bin pong-rebuild-data
```

## Play pong

```sh
cargo m run --bin runtime-srv --features=standalone -- --manifest=examples/pong/data/runtime/game.manifest --root-asset="(1d9ddd99aad89045,b3440a7c-ba07-5628-e7f8-bb89ed5de900)"
```

## Edit the pong sample in the editor

```sh
cargo m run --bin editor-srv -- --project-root=./target/data/workspaces/pong --repository-name=examples-pong --manifest=examples/pong/data/runtime/game.manifest --scene "/scene.ent" --build-output-database-address=./target/output_db
cargo m run --bin editor-client
```

## Data exploration

```sh
cargo m run --bin data-scrape -- configure --project examples/pong/data --output temp/
cargo m run --bin data-scrape -- asset examples/pong/data/temp
```
