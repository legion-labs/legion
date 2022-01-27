import { Entries } from "@/lib/hierarchyTree";

export const fakeFileSystemEntries = new Entries([
  {
    name: "Archives",
    index: 0,
    item: 1,
    depth: 0,
    subEntries: [
      {
        name: "quarterly-results.zip",
        index: 1,
        item: 2,
        depth: 1,
        subEntries: null,
      },
      {
        name: "data.zip",
        index: 2,
        item: 3,
        depth: 1,
        subEntries: null,
      },
    ],
  },
  {
    name: "Assets",
    index: 3,
    item: 4,
    depth: 0,
    subEntries: [
      {
        name: "Nature",
        index: 4,
        item: 5,
        depth: 1,
        subEntries: [
          {
            name: "sand.png",
            index: 5,
            item: 6,
            depth: 2,
            subEntries: null,
          },
          {
            name: "tree.png",
            index: 6,
            item: 7,
            depth: 2,
            subEntries: null,
          },
          {
            name: "rock.jpg",
            index: 7,
            item: 8,
            depth: 2,
            subEntries: null,
          },
        ],
      },
      {
        name: "City",
        index: 8,
        item: 9,
        depth: 1,
        subEntries: [
          {
            name: "building.jpg",
            index: 9,
            item: 10,
            depth: 2,
            subEntries: null,
          },
          {
            name: "street.png",
            index: 10,
            item: 11,
            depth: 2,
            subEntries: null,
          },
        ],
      },
      {
        name: "other.jpg",
        index: 11,
        item: 12,
        depth: 1,
        subEntries: null,
      },
      {
        name: "concept.png",
        index: 12,
        item: 13,
        depth: 1,
        subEntries: null,
      },
    ],
  },
  {
    name: "TODO.pdf",
    index: 13,
    item: 14,
    depth: 0,
    subEntries: null,
  },
]);
