import { suite } from "../benchmark";

import { Entries, Entry, isEntry } from "@/lib/hierarchyTree";
import resources from "../resources/resourcesResponse.json";

// Dumb polyfill for `getRandomValues`
global.crypto = {
  getRandomValues: (buf: ArrayBufferView) => {
    return buf;
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
} as any;

export const resourcesSuite = suite("Entries.fromArray", (bench) => {
  bench.add(
    "Transforms resources into hierarchy tree entries",
    { iter: 10_000 },
    () => {
      return () => {
        Entries.fromArray(resources);
      };
    }
  );

  bench.add("Compute size of entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);

    return () => {
      entries.recalculateSize();
    };
  });

  bench.add("Find early entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);

    return () => {
      entries.find((entry) => entry.name === "DebugCube1");
    };
  });

  bench.add("Find late entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);

    return () => {
      entries.find((entry) => entry.name === "ground.mat");
    };
  });

  bench.add("Update early entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);

    return () => {
      entries.update((entry) =>
        entry.name === "DebugCube1" ? { ...entry, name: "DebugCube10" } : null
      );
    };
  });

  bench.add("Update late entry in entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);

    return () => {
      entries.update((entry) =>
        entry.name === "DebugCube1" ? { ...entry, name: "ground.mat" } : null
      );
    };
  });

  bench.add("Remove early entry from entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);

    const entry = entries.entries[0].subEntries[0];

    if (!isEntry(entry)) {
      throw new Error("Entry was not a proper entry");
    }

    return () => {
      entries.remove(entry);
    };
  });

  bench.add("Remove late entry from entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);

    const entry = entries.entries[3].subEntries[0].subEntries[4];

    if (!isEntry(entry)) {
      throw new Error("Entry was not a proper entry");
    }

    return () => {
      entries.remove(entry);
    };
  });

  bench.add("Insert item early in entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);
    const entry = entries.getFromIndex(2) as Entry<{ path: string }>;

    return () => {
      entries.insert({
        ...entry.item,
        path: `${entry.item.path} - Copy`,
        id: "id",
        version: 1,
      });
    };
  });

  bench.add("Insert item late in entries", { iter: 10_000 }, () => {
    const entries = Entries.fromArray(resources);
    const entry = entries.getFromIndex(entries.size() - 2) as Entry<{
      path: string;
    }>;

    return () => {
      entries.insert({
        ...entry.item,
        path: `${entry.item.path} - Copy`,
        id: "id",
        version: 1,
      });
    };
  });
});

export const bigResourcesSuite = suite(
  "Entries.fromArray - big resources",
  (bench) => {
    const bigResources: { path: string; id: string; version: number }[] = [];

    for (let i = 0; i < 1_000; i++) {
      bigResources.push({
        id: "(07dd9f5d1793ed64,48909c46-ad4f-4d6b-a522-2e16e81ba082)",
        path: `/world${i}/sample_1${i}/ground.mat${i}`,
        version: 1,
      });
    }

    bench.add(
      "Transforms resources into hierarchy tree entries - big resources",
      { iter: 100 },
      () => {
        return () => {
          Entries.fromArray(bigResources);
        };
      }
    );

    bench.add("Compute size of entries - big resources", { iter: 100 }, () => {
      const entries = Entries.fromArray(bigResources);

      return () => {
        entries.recalculateSize();
      };
    });

    bench.add(
      "Find early entry in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.fromArray(bigResources);

        return () => {
          entries.find((entry) => entry.name === "DebugCube1");
        };
      }
    );

    bench.add(
      "Find late entry in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.fromArray(bigResources);

        return () => {
          entries.find((entry) => entry.name === "ground.mat");
        };
      }
    );

    bench.add(
      "Update early entry in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.fromArray(bigResources);

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
        const entries = Entries.fromArray(bigResources);

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
        const entries = Entries.fromArray(bigResources);

        const entry = entries.entries[0].subEntries[0].subEntries[0];

        if (!isEntry(entry)) {
          throw new Error("Entry was not a proper entry");
        }

        return () => {
          entries.remove(entry);
        };
      }
    );

    bench.add(
      "Remove late entry from entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.fromArray(bigResources);

        const entry = entries.entries[100].subEntries[0].subEntries[0];

        if (!isEntry(entry)) {
          throw new Error("Entry was not a proper entry");
        }

        return () => {
          entries.remove(entry);
        };
      }
    );

    bench.add(
      "Insert item early in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.fromArray(bigResources);
        const entry = entries.getFromIndex(2) as Entry<{ path: string }>;

        return () => {
          entries.insert({
            ...entry.item,
            path: `${entry.item.path} - Copy`,
            id: "id",
            version: 1,
          });
        };
      }
    );

    bench.add(
      "Insert item late in entries - big resources",
      { iter: 100 },
      () => {
        const entries = Entries.fromArray(bigResources);
        const entry = entries.getFromIndex(entries.size() - 2) as Entry<{
          path: string;
        }>;

        return () => {
          entries.insert({
            ...entry.item,
            path: `${entry.item.path} - Copy`,
            id: "id",
            version: 1,
          });
        };
      }
    );
  }
);
