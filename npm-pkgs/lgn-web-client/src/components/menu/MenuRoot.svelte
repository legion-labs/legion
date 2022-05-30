<script lang="ts">
  import Icon from "@iconify/svelte";

  import clickOutside from "../../actions/clickOutside";
  import MenuItem from "./MenuItem.svelte";
  import type { MenuContextStore } from "./lib/MenuContextStore";
  import type { MenuItemDescription } from "./lib/MenuItemDescription";

  export let menuContext: MenuContextStore;
  export let desc: MenuItemDescription;
  export let enableHover: boolean;
  export let container: HTMLElement;
  export let parent: HTMLElement;

  let menuWidth: number;
  let itemsWidth: number;
  let parentWidth: number;
  let overflow: boolean;

  $: overflow = parentWidth + itemsWidth > container?.clientWidth;
  $: selected = $menuContext.current === desc;
  $: displayable = desc.children?.some((c) => c.visible) ?? false;

  function onClick() {
    menuContext.onRootClick(desc);
    parentWidth = parent.offsetLeft;
  }
</script>

<div
  hidden={!displayable}
  class:flex={displayable}
  class:bg-gray-400={enableHover && selected}
  class={`menu ${enableHover ? "hover:bg-gray-500" : ""}`}
  use:clickOutside
  on:click-outside={menuContext.close}
  on:mouseenter={() => menuContext.mouseEnter(desc)}
  on:click|stopPropagation={onClick}
>
  <div class="menu-title" bind:clientWidth={menuWidth}>
    {#if desc.icon}
      <div class="self-center">
        <Icon icon={desc.icon} />
      </div>
    {/if}
    {#if desc.title}
      <div class="self-center">
        {desc.title}
      </div>
    {/if}
  </div>
  <div
    class="menu-dropdown"
    class:electron={window.isElectron}
    class:hidden={$menuContext.current !== desc}
  >
    <div
      class="menu-dropdown-items"
      bind:clientWidth={itemsWidth}
      style={`${overflow ? `right:${-menuWidth / 1.5}px` : ""}`}
    >
      {#if desc.children}
        {#each desc.children as item}
          {#if item.visible}
            <MenuItem desc={item} {menuContext} />
          {/if}
        {/each}
      {/if}
    </div>
  </div>
</div>

<style lang="postcss">
  .menu {
    @apply items-center cursor-pointer h-full;
  }

  .menu-title {
    @apply px-2 flex;
  }

  .menu-dropdown {
    @apply absolute top-7 rounded-b-sm z-[10];
  }

  .menu-dropdown.electron {
    @apply top-9;
  }

  .menu-dropdown-items {
    box-shadow: 0px 2px 10px -2px rgba(0, 0, 0, 0.7);
    @apply bg-gray-800 py-1 rounded-b-sm absolute z-10;
  }

  .left {
    @apply -right-full;
  }
</style>
