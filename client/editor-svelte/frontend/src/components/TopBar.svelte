<script lang="ts">
  import clickOutside from "@/actions/clickOutside";
  import {
    default as topBarMenu,
    Id as TopBarMenuId,
    menus as topBarMenus,
  } from "@/stores/topBarMenu";

  export let documentTitle: string | null = null;

  const onMenuMouseEnter = (id: TopBarMenuId) => {
    // We set the topBarMenu value (and therefore open said menu dropdown)
    // only when a menu is open
    $topBarMenu && topBarMenu.set(id);
  };

  const onMenuClick = (id: TopBarMenuId) => {
    // Simple menu dropdown display toggle
    $topBarMenu ? topBarMenu.close() : topBarMenu.set(id);
  };

  const onMenuItemClick = () => {
    // When a user clicks on a menu dropdown item, we just close the menu
    topBarMenu.close();
    console.log("Executed");
  };

  const closeMenu = () => {
    if ($topBarMenu) {
      topBarMenu.close();
    }
  };
</script>

<div class="root">
  <div use:clickOutside={closeMenu} class="menus">
    <div class="brand">Legion</div>
    {#each topBarMenus as menu (menu.id)}
      <div
        data-testid="menu-{menu.id}"
        class="menu"
        class:bg-gray-400={$topBarMenu === menu.id}
        on:mouseenter={() => onMenuMouseEnter(menu.id)}
        on:click|capture={() => onMenuClick(menu.id)}
      >
        <div class="menu-title">
          {menu.title}
        </div>
        <div
          data-testid="dropdown-{menu.id}"
          class="menu-dropdown"
          class:hidden={$topBarMenu !== menu.id}
        >
          <div class="menu-dropdown-items">
            {#each [`Foo ${menu.title}`, `Bar ${menu.title}`, `Baz ${menu.title}`] as menuItemTitle}
              <div class="menu-dropdown-item" on:click={onMenuItemClick}>
                {menuItemTitle}
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/each}
  </div>
  <div class="document-title">
    {#if documentTitle}
      {documentTitle}
    {:else}
      Untitled document
    {/if}
  </div>
  <div class="filler" />
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row justify-between space-x-2;
  }

  .brand {
    @apply flex items-center italic px-2;
  }

  .menus {
    @apply flex flex-row flex-1 h-7 space-x-1 text-sm;
  }

  .menu {
    @apply flex items-center cursor-pointer z-10 hover:bg-gray-400;
  }

  .menu-title {
    @apply px-2;
  }

  .menu-dropdown {
    @apply absolute top-7;
  }

  .menu-dropdown-items {
    @apply bg-gray-800 py-1 bg-opacity-90;
  }

  .menu-dropdown-item {
    @apply hover:bg-gray-400 cursor-pointer px-6 py-0.5;
  }

  .root {
    @apply flex flex-row justify-center items-center flex-1 whitespace-nowrap;
  }

  .filler {
    @apply flex-1;
  }
</style>
