<script lang="ts">
  import clickOutside from "@/directives/clickOutside";

  // Props
  export let documentTitle: string | null = null;

  // Types

  // Obviously not meant to be used as is in production
  // as the menu might become dynamic at one point
  type Id = typeof menus[number]["id"];

  // Values
  let openedMenu: Id | null = null;

  const menus = [
    { id: 1, title: "File" },
    { id: 2, title: "Edit" },
    { id: 3, title: "Layer" },
    { id: 4, title: "Document" },
    { id: 5, title: "View" },
    { id: 6, title: "Help" },
  ] as const;

  // Callbacks

  const onMenuMouseEnter = (id: Id) =>
    // We set the openedMenu value (and therefore open said menu dropdown)
    // only when a menu is open
    openedMenu && (openedMenu = id);

  const onMenuClick = (id: Id) => {
    // Simple menu dropdown display toggle
    openedMenu = openedMenu ? null : id;
  };

  const onMenuItemClick = () => {
    // When a user clicks on a menu dropdown item, we just close the menu
    openedMenu = null;
    console.log("Executed");
  };

  const closeMenu = () => {
    if (openedMenu) {
      openedMenu = null;
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
        class="flex items-center hover:bg-gray-400 cursor-pointer"
        class:bg-gray-400={openedMenu === menu.id}
        on:mouseenter={() => onMenuMouseEnter(menu.id)}
        on:click|capture={() => onMenuClick(menu.id)}
      >
        <div class="px-2">
          {menu.title}
        </div>
        <div class="absolute top-7" class:hidden={openedMenu !== menu.id}>
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
