export type Options = { iter: number };

type Result =
  | {
      type: "result";
      name: string;
      total: number;
      average: number;
      iter: number;
    }
  | { type: "skipped"; name: string };

type BenchmarkThunk = () =>
  | Promise<() => Promise<void> | void>
  | (() => Promise<void> | void);

class Benchmark {
  #benches: {
    name: string;
    options: Options;
    thunk: BenchmarkThunk;
    skipped?: boolean;
  }[] = [];

  skip(name: string, options: Options, thunk: BenchmarkThunk) {
    this.#benches.push({ name, options, thunk, skipped: true });
  }

  add(name: string, options: Options, thunk: BenchmarkThunk) {
    this.#benches.push({ name, options, thunk });
  }

  async *run(): AsyncGenerator<Result> {
    // TODO: Could be done in parallel
    for (const { name, options, thunk, skipped = false } of this.#benches) {
      if (skipped) {
        yield { type: "skipped", name };

        continue;
      }

      let total = 0n;

      // TODO: Could be done in parallel
      for (let i = 0; i < options.iter; i++) {
        const benchmark = await thunk();

        const start: bigint = process.hrtime.bigint();

        await benchmark();

        const end: bigint = process.hrtime.bigint();

        // We lose a bit of precision just so the bigint doesn't overflow
        total += (end - start) / 1_000n;
      }

      const totalMs = Number(total) / 1_000;

      yield {
        type: "result",
        name,
        total: totalMs,
        average: totalMs / options.iter,
        iter: options.iter,
      };
    }
  }

  dispose() {
    this.#benches = [];
  }
}

export function suite(
  name: string,
  f: (benchmark: Benchmark) => void
): () => Promise<void> {
  const benchmark = new Benchmark();

  f(benchmark);

  return async function () {
    let prompted = false;

    for await (const result of benchmark.run()) {
      if (!prompted) {
        console.log(`> Running suite ${name}`);

        prompted = true;
      }

      switch (result.type) {
        case "result": {
          console.log(
            `${result.name}: \n\ttotal: ${result.total.toFixed(
              3
            )}ms\n\taverage: ${result.average.toFixed(3)}ms\n\titeration: ${
              result.iter
            }`
          );

          break;
        }

        case "skipped": {
          console.log(`Skipped ${result.name}`);

          break;
        }
      }
    }

    benchmark.dispose();
  };
}

export async function run(suites: (() => Promise<void>)[]): Promise<void> {
  return (
    Promise.all(suites.map((suite) => suite()))
      // Ignores results
      .then(() => undefined)
      .catch(console.error)
  );
}
