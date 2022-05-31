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

<div class="menu-dropdown-item" on:click|stopPropagation={onClick}>
  {desc.title}
</div>

<style lang="postcss">
  .menu-dropdown-item {
    @apply h-8 px-3 pt-1;
    @apply text-menu-text-enabled hover:text-menu-text-hovered;
    @apply bg-menu-default hover:bg-menu-hovered;
    @apply cursor-pointer whitespace-nowrap;
  }
</style>
