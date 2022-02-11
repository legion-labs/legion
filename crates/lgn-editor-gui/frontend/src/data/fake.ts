import { Entries } from "@/lib/hierarchyTree";

export const fakeFileSystemEntries = new Entries([
  {
    name: "Archives",
    index: 0,
    item: 1,
    subEntries: [
      {
        name: "quarterly-results.zip",
        index: 1,
        item: 2,
        subEntries: null,
      },
      {
        name: "data.zip",
        index: 2,
        item: 3,
        subEntries: null,
      },
    ],
  },
  {
    name: "Assets",
    index: 3,
    item: 4,
    subEntries: [
      {
        name: "Nature",
        index: 4,
        item: 5,
        subEntries: [
          {
            name: "sand.png",
            index: 5,
            item: 6,
            subEntries: null,
          },
          {
            name: "tree.png",
            index: 6,
            item: 7,
            subEntries: null,
          },
          {
            name: "rock.jpg",
            index: 7,
            item: 8,
            subEntries: null,
          },
        ],
      },
      {
        name: "City",
        index: 8,
        item: 9,
        subEntries: [
          {
            name: "building.jpg",
            index: 9,
            item: 10,
            subEntries: null,
          },
          {
            name: "street.png",
            index: 10,
            item: 11,
            subEntries: null,
          },
        ],
      },
      {
        name: "other.jpg",
        index: 11,
        item: 12,
        subEntries: null,
      },
      {
        name: "concept.png",
        index: 12,
        item: 13,
        subEntries: null,
      },
    ],
  },
  {
    name: "TODO.pdf",
    index: 13,
    item: 14,
    subEntries: null,
  },
]);
