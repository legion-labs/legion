export type MenuItemDescription = {
  title: string;
  action?: () => void;
  children?: MenuItemDescription[];
};
