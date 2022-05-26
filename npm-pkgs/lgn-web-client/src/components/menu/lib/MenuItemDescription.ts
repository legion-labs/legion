export type MenuItemDescription = {
  title?: string;
  icon?: string;
  visible: boolean;
  action?: () => void;
  children?: MenuItemDescription[];
};
