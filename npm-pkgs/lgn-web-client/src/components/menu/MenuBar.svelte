<script lang="ts">
  import MenuRoot from "./MenuRoot.svelte";
  import { getMenuContextStore } from "./lib/MenuContextStore";
  import type { MenuItemDescription } from "./lib/MenuItemDescription";

  const menuContext = getMenuContextStore();

  export let items: MenuItemDescription[] = [];
  export let enableHover = true;
</script>

{#each items as menu (menu.title)}
  <MenuRoot desc={menu} {menuContext} {enableHover} />
{/each}
