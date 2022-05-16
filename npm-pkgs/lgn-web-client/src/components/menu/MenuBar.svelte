<script lang="ts">
  import clickOutside from "../../actions/clickOutside";
  import { getMenuContextStore } from "./lib/MenuContextStore";
  import type { MenuItemDescription } from "./lib/MenuItemDescription";
  import MenuRoot from "./MenuRoot.svelte";

  const menuContext = getMenuContextStore();

  export let items: MenuItemDescription[];
</script>

<div use:clickOutside on:click-outside={menuContext.close} />
{#each items as menu (menu.title)}
  <MenuRoot desc={menu} {menuContext} />
{/each}
