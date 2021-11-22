<script lang="ts">
  import clickOutside from "@/directives/clickOutside";
  import {
    set as setMenuId,
    openedMenuId,
    close as closeTopBarMenu,
    menus,
    Id as MenuId,
  } from "@/stores/topBarMenu";

  // Props

  export let documentTitle: string | null = null;

  // Callbacks

  const onMenuMouseEnter = (id: MenuId) => {
    // We set the openedMenu value (and therefore open said menu dropdown)
    // only when a menu is open
    $openedMenuId && setMenuId(id);
  };

  const onMenuClick = (id: MenuId) => {
    // Simple menu dropdown display toggle
    $openedMenuId ? closeTopBarMenu() : setMenuId(id);
  };

  const onMenuItemClick = () => {
    // When a user clicks on a menu dropdown item, we just close the menu
    closeTopBarMenu();
    console.log("Executed");
  };

  const closeMenu = () => {
    if ($openedMenuId) {
      closeTopBarMenu();
    }
  };
</script>

<div class="flex flex-row justify-between space-x-2">
  <div
    use:clickOutside
    on:click-outside={closeMenu}
    class="flex flex-row flex-1 h-7 space-x-1 text-sm"
  >
    <div class="flex items-center italic px-2">Legion</div>
    {#each menus as menu}
      <div
        data-testid="menu-{menu.id}"
        class="flex items-center hover:bg-gray-400 cursor-pointer"
        class:bg-gray-400={$openedMenuId === menu.id}
        on:mouseenter={() => onMenuMouseEnter(menu.id)}
        on:click|capture={() => onMenuClick(menu.id)}
      >
        <div class="px-2">
          {menu.title}
        </div>
        <div
          data-testid="dropdown-{menu.id}"
          class="absolute top-7"
          class:hidden={$openedMenuId !== menu.id}
        >
          <div class="bg-gray-800 py-1 bg-opacity-90">
            {#each [`Foo ${menu.title}`, `Bar ${menu.title}`, `Baz ${menu.title}`] as menuItemTitle}
              <div
                class="cursor-pointer hover:bg-gray-400 px-6 py-0.5"
                on:click={onMenuItemClick}
              >
                {menuItemTitle}
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/each}
  </div>
  <div
    class="flex flex-row justify-center items-center flex-1 whitespace-nowrap"
  >
    {#if documentTitle}
      {documentTitle}
    {:else}
      Untitled document
    {/if}
  </div>
  <div class="flex-1" />
</div>
