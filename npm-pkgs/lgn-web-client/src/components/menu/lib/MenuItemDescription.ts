export type MenuItemDescription = {
  title?: string;
  icon?: string;
  hidden?: boolean;
  action?: () => void;
  children?: MenuItemDescription[];
};
