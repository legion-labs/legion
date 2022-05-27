<script lang="ts">
  import MenuRoot from "./MenuRoot.svelte";
  import { getMenuContextStore } from "./lib/MenuContextStore";
  import type { MenuItemDescription } from "./lib/MenuItemDescription";

  const menuContext = getMenuContextStore();

  export let items: MenuItemDescription[] = [];
  export let enableHover = true;
  export let container: HTMLElement;

  let menuRoot: HTMLElement;
</script>

<div bind:this={menuRoot}>
  {#each items as menu (menu.title)}
    <MenuRoot
      desc={menu}
      parent={menuRoot}
      {menuContext}
      {enableHover}
      {container}
    />
  {/each}
</div>
