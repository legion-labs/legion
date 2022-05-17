<script lang="ts">
  import { getMenuContextStore } from "./lib/MenuContextStore";
  import type { MenuItemDescription } from "./lib/MenuItemDescription";
  import MenuRoot from "./MenuRoot.svelte";

  const menuContext = getMenuContextStore();

  export let items: MenuItemDescription[];
</script>

{#each items as menu (menu.title)}
  <MenuRoot desc={menu} {menuContext} />
{/each}
