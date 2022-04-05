# Pong example

## Launching

```sh
cargo m run --bin runtime-srv --features=standalone -- --project=examples/pong/data --root="(1d9ddd99aad89045,b3440a7c-ba07-5628-e7f8-bb89ed5de900)"
```

## Editing data

```sh
cargo m run --bin editor-srv -- --project=./target/data/workspaces/pong/ --origin=../../../../../examples/pong/data/remote/ --build-db=./target/data/build-db/ --scene "/scene.ent"
cargo m run --bin editor-client
```

## Data regeneration

```sh
cargo m run --bin pong-rebuild-data
```

## Data exploration

```sh
cargo m run --bin data-scrape -- configure --project examples/pong/data --output temp/
cargo m run --bin data-scrape -- asset examples/pong/data/temp
```
