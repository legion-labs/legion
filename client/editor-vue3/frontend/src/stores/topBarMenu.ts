import { defineStore } from "pinia";

export const menus = [
  { id: 1, title: "File" },
  { id: 2, title: "Edit" },
  { id: 3, title: "Layer" },
  { id: 4, title: "Document" },
  { id: 5, title: "View" },
  { id: 6, title: "Help" },
] as const;

// Obviously not meant to be used as is in production
// as the menu might become dynamic at one point
export type Id = typeof menus[number]["id"];

export const useTopBarMenu = defineStore("topBarMenu", {
  state() {
    return { openedMenuId: null as Id | null };
  },
  getters: {
    isOpen(): boolean {
      return !!this.openedMenuId;
    },
  },
  actions: {
    close() {
      this.openedMenuId = null;
    },
    set(id: Id) {
      this.openedMenuId = id;
    },
  },
});
