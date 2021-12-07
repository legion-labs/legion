# Analytics - Client

_You can take a look at the [client documentation](./frontend/README.md) too._

## Getting started

You can simply run:

```bash
cargo run
```

And the application should start.

## Starting the web app in dev mode


```
cd frontend
yarn install
yarn dev
```

Or from the root of the monorepo:

```
yarn analytics:dev
```

## validations: tsc && svelte-check

```
yarn workspace analytics-web run svelte:check
```
