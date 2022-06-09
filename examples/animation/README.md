# Animation Example

## Data Regeneration

If it is your first time running the Animation Example or if you changed the data in the main.rs, you must generate the data.

You can run the task "animation-rebuild-data" or run the following command:

```sh
cargo m run --bin animation-rebuild-data
```
## Launching

In order to visualize the animation example, you must launch the runtime server.

To do so, you can run the following command:
```sh
cargo m run --bin runtime-srv --features=standalone -- --manifest=examples/animation/data/runtime/game.manifest --root-asset="(1d9ddd99aad89045,1fa058cb-5877-5ffe-dcb7-1f364a804a8f)"
```

After the launch, to see the skeleton bones moving:

- Press the "m" key to open the debug display menu
- Click on "Animation options"
- Tick the "Show animation skeleton bones" box

You should now see the animation example. 

The example is looping through a simple animation clip.

## Editing Data

```sh
cargo m run --bin editor-srv -- --project-root=./target/data/workspaces/animation --repository-name=examples-animation --manifest=examples/animation/data/runtime/game.manifest --scene "/scene.ent" --build-output-database-address=./target/output_db
cargo m run --bin editor-client
```

## Data Exploration

```sh
cargo m run --bin data-scrape -- configure --project examples/animation/data --output temp/
cargo m run --bin data-scrape -- asset examples/animation/data/temp
```