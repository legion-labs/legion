<script lang="ts">
  import clickOutside from "../../actions/clickOutside";
  import type { MenuContextStore } from "./lib/MenuContextStore";
  import type { MenuItemDescription } from "./lib/MenuItemDescription";
  import MenuItem from "./MenuItem.svelte";

  export let menuContext: MenuContextStore;
  export let desc: MenuItemDescription;
</script>

<div
  class="menu"
  class:bg-gray-400={$menuContext.current === desc}
  use:clickOutside
  on:click-outside={menuContext.close}
  on:mouseenter={() => menuContext.mouseEnter(desc)}
  on:click|capture={() => menuContext.onRootClick(desc)}
>
  <div class="menu-title">
    {desc.title}
  </div>
  <div
    class="menu-dropdown"
    class:electron={window.isElectron}
    class:hidden={$menuContext.current !== desc}
  >
    <div class="menu-dropdown-items">
      {#if desc.children}
        {#each desc.children as item}
          <MenuItem desc={item} {menuContext} />
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .menu {
    @apply hidden sm:flex items-center cursor-pointer z-40 hover:bg-gray-500;
  }

  .menu-title {
    @apply px-2;
  }

  .menu-dropdown {
    @apply absolute top-7 rounded-b-sm shadow-lg shadow-gray-800;
  }

  .menu-dropdown.electron {
    @apply top-9;
  }

  .menu-dropdown-items {
    @apply bg-gray-800 py-1 rounded-b-sm;
  }
</style>
