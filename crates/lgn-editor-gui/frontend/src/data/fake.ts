import { Entries } from "@/lib/hierarchyTree";

export const fakeFileSystemEntries: Entries<number> = {
  Archives: {
    item: 1,
    entries: {
      "quarterly-results.zip": {
        item: 2,
      },
      "data.zip": {
        item: 3,
      },
    },
  },
  Assets: {
    item: 4,
    entries: {
      Nature: {
        item: 6,
        entries: {
          "sand.png": {
            item: 7,
          },
          "tree.png": {
            item: 8,
          },
          "rock.jpg": {
            item: 9,
          },
        },
      },
      City: {
        item: 10,
        entries: {
          "building.jpg": {
            item: 11,
          },
          "street.png": {
            item: 12,
          },
        },
      },
      "other.jpg": {
        item: 13,
      },
      "concept.png": {
        item: 14,
      },
    },
  },
  "TODO.pdf": {
    item: 15,
  },
};
