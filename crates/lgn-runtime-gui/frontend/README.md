# Legion Runtime - Client

_You can take a look at the [Client documentation](../README.md) too._

## Resources

First, you might want to read the documentation for the libraries the client uses:

- [Svelte](https://svelte.dev/) - The UI framework itself
- [TypeScript](https://www.typescriptlang.org/) - The language the application is written in
- [Tailwind](https://tailwindcss.com/) - An "atomic" css library that makes prototyping a breeze
- [Vite](vitejs.dev/) - The application "runner" and bundler you will love
- [Jest](https://jestjs.io/) - To test the client

We also use [ESLint](https://eslint.org/) and [Prettier](https://prettier.io/) to keep our codebase as clean as posible.

## Getting started

Make sure to install the dependencies

```bash
pnpm install
```

You can now run the application:

```
pnpm start
```

The development server on will be accessible on [http://localhost:3000](http://localhost:3000).

## Keeping the code clean

You should regularly check that the code is clean and properly formatted:

```
pnpm lint # pnpm lint:fix to fix the errors when possible
pnpm fmt:check # pnpm fmt to format the code
pnpm svelte:check # TypeScript and Svelte code checkers
```

## Tests

You can run the tests by simply typing:

```
pnpm test
```

## Production

You can build the application for production using this command:

```bash
pnpm build
```
