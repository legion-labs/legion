import { suite } from "../benchmark";

import { Entries } from "@/lib/hierarchyTree";
import resources from "../resources/resourcesResponse.json";

export const resourcesSuite = suite("Entries.unflatten", (bench) => {
  bench.add("Unflatten entries", { iter: 10_000 }, () => {
    return () => {
      Entries.unflatten(resources, Symbol);
    };
  });

  bench.add("Compute size of entries", { iter: 10_000 }, () => {
    const entries = Entries.unflatten(resources, Symbol);

    return () => {
      entries.size;
    };
  });

  bench.add("Find early entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.unflatten(resources, Symbol);

    return () => {
      entries.find((entry) => entry.name === "DebugCube1");
    };
  });

  bench.add("Find late entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.unflatten(resources, Symbol);

    return () => {
      entries.find((entry) => entry.name === "ground.mat");
    };
  });

  bench.add("Update early entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.unflatten(resources, Symbol);

    return () => {
      entries.update((entry) =>
        entry.name === "DebugCube1" ? { ...entry, name: "DebugCube10" } : null
      );
    };
  });

  bench.add("Update late entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.unflatten(resources, Symbol);

    return () => {
      entries.update((entry) =>
        entry.name === "DebugCube1" ? { ...entry, name: "ground.mat" } : null
      );
    };
  });

  bench.add("Remove early entry from entries", { iter: 10_000 }, () => {
    const entries = Entries.unflatten(resources, Symbol);

    return () => {
      entries.remove(entries.entries[2]);
    };
  });

  bench.add("Remove late entry from entries", { iter: 10_000 }, () => {
    const entries = Entries.unflatten(resources, Symbol);

    return () => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      entries.remove(entries.entries[3].subEntries![0].subEntries![4]);
    };
  });
});

export const bigResourcesSuite = suite(
  "Entries.unflatten - big resources",
  (bench) => {
    const bigResources: { path: string; id: string; version: number }[] = [];

    for (let i = 0; i < 1_000; i++) {
      bigResources.push({
        id: "(07dd9f5d1793ed64,48909c46-ad4f-4d6b-a522-2e16e81ba082)",
        path: `/world${i}/sample_1${i}/ground.mat${i}`,
        version: 1,
      });
    }

    bench.add("Unflatten entries - big resources", { iter: 100 }, () => {
      return () => {
        Entries.unflatten(bigResources, Symbol);
      };
    });

    bench.add("Compute size of entries - big resources", { iter: 100 }, () => {
      const entries = Entries.unflatten(bigResources, Symbol);

      return () => {
        entries.size;
      };
    });

    bench.add(
      "Find early entry in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.unflatten(bigResources, Symbol);

        return () => {
          entries.find((entry) => entry.name === "DebugCube1");
        };
      }
    );

    bench.add(
      "Find late entry in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.unflatten(bigResources, Symbol);

        return () => {
          entries.find((entry) => entry.name === "ground.mat");
        };
      }
    );

    bench.add(
      "Update early entry in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.unflatten(bigResources, Symbol);

        return () => {
          entries.update((entry) =>
            entry.name === "DebugCube1"
              ? { ...entry, name: "DebugCube10" }
              : null
          );
        };
      }
    );

    bench.add(
      "Update late entry in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.unflatten(bigResources, Symbol);

        return () => {
          entries.update((entry) =>
            entry.name === "DebugCube1"
              ? { ...entry, name: "ground.mat" }
              : null
          );
        };
      }
    );

    bench.add(
      "Remove early entry from entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.unflatten(bigResources, Symbol);

        return () => {
          entries.remove(entries.entries[2]);
        };
      }
    );

    bench.add(
      "Remove late entry from entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.unflatten(bigResources, Symbol);

        return () => {
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          entries.remove(entries.entries[3].subEntries![0].subEntries![4]);
        };
      }
    );
  }
);
