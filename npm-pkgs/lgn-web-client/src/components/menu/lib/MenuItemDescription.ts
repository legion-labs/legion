export type MenuItemDescription = {
  title: string;
  // note : main game view might no create a new window if already open
  action?: () => void;
  children?: MenuItemDescription[];
};

export const mainMenuItemDescriptions: MenuItemDescription[] = [
  {
    title: "Test 1",
    children: [
      {
        title: "Child Test 1.1",
        children: [
          {
            title: "Sub",
          },
          { title: "Dub" },
        ],
      },
      { title: "Child Test 1.2" },
    ],
  },
  {
    title: "Test 2",
    children: [
      {
        title: "Child Test 2.1",
      },
      { title: "Child Test 2.2" },
    ],
  },
];
