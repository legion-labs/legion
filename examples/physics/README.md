# Physics example

## Launching

```sh
cargo m run --bin runtime-srv -- --project=examples/physics/data --root="(1d9ddd99aad89045,1fa058cb-5877-5ffe-dcb7-1f364a804a8f)" --standalone --physics-debugger
```

## Editing data

```sh
cargo m run --bin editor-srv -- --project=examples/physics/data --scene "/scene.ent"
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
