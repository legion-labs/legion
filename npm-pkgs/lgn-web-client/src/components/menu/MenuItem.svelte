<script lang="ts">
  import type { MenuContextStore } from "./lib/MenuContextStore";
  import type { MenuItemDescription } from "./lib/MenuItemDescription";

  export let desc: MenuItemDescription;
  export let menuContext: MenuContextStore;

  function onClick() {
    menuContext.close();

    if (desc.action) {
      desc.action();
    }
  }
</script>

<div class="menu-dropdown-item" on:click={onClick}>
  {desc.title}
</div>

<style lang="postcss">
  .menu-dropdown-item {
    @apply hover:bg-gray-500 cursor-pointer px-6 py-0.5 whitespace-nowrap;
  }
</style>
