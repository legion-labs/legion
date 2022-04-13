interface Window {
  isElectron?: boolean;
  electron?: {
    toggleMaximizeMainWindow(this: void): void;
    minimizeMainWindow(this: void): void;
    closeMainWindow(this: void): void;
  };
}
