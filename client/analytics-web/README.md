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
pnpm install
pnpm dev
```

Or from the root of the monorepo:

```
pnpm analytics:dev
```

## Starting the analytics server

The web app in only a client to the analytics gRPC server. You can execute `legion\server\analytics-srv\start-test-server.bat` to start the server using dummy validation data.

## validations: tsc && svelte-check

```
pnpm -C client/analytics-web/frontend svelte:check
```
