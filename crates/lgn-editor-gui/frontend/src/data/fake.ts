export const fakeFileSystemEntries = [
  {
    type: "directory" as const,
    name: "Archives",
    entries: [
      {
        type: "file" as const,
        name: "quarterly-results.zip",
        item: 1,
      },
      {
        type: "file" as const,
        name: "data.zip",
        item: 2,
      },
    ],
  },
  {
    type: "directory" as const,
    name: "Assets",
    entries: [
      {
        type: "directory" as const,
        name: "Nature",
        entries: [
          {
            type: "file" as const,
            name: "sand.png",
            item: 3,
          },
          {
            type: "file" as const,
            name: "tree.png",
            item: 4,
          },
          {
            type: "file" as const,
            name: "rock.jpg",
            item: 5,
          },
        ],
      },
      {
        type: "directory" as const,
        name: "City",
        entries: [
          {
            type: "file" as const,
            name: "building.jpg",
            item: 6,
          },
          {
            type: "file" as const,
            name: "street.png",
            item: 7,
          },
        ],
      },
      {
        type: "file" as const,
        name: "other.jpg",
        item: 8,
      },
      {
        type: "file" as const,
        name: "concept.png",
        item: 9,
      },
    ],
  },
  {
    type: "file" as const,
    name: "TODO.pdf",
    item: 10,
  },
];
