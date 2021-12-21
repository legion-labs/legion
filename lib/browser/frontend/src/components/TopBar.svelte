<script lang="ts">
  import { invoke } from "@tauri-apps/api";
  import topBarMenu, {
    Id as TopBarMenuId,
    menus as topBarMenus,
  } from "../stores/topBarMenu";
  import { createAwsCognitoTokenCache } from "../lib/auth";
  import userInfo from "../stores/userInfo";
  import log from "../lib/log";
  import clickOutside from "../actions/clickOutside";

  export let documentTitle: string | null = null;

  const { data: userInfoData } = userInfo;

  $: userInitials =
    $userInfoData && $userInfoData.given_name && $userInfoData.family_name
      ? `${$userInfoData.given_name[0]}${$userInfoData.family_name[0]}`
      : // TODO: Use an icon
        "Me";

  function onMenuMouseEnter(id: TopBarMenuId) {
    // We set the topBarMenu value (and therefore open said menu dropdown)
    // only when a menu is open
    $topBarMenu && topBarMenu.set(id);
  }

  function onMenuClick(id: TopBarMenuId) {
    // Simple menu dropdown display toggle
    $topBarMenu ? topBarMenu.close() : topBarMenu.set(id);
  }

  function onMenuItemClick(title: string) {
    log.debug("layout:topbar", `Clicked on item ${title}`);
    // When a user clicks on a menu dropdown item, we just close the menu for now
    topBarMenu.close();
  }

  function closeMenu() {
    if ($topBarMenu) {
      topBarMenu.close();
    }
  }

  async function authenticate() {
    if (window.__TAURI__) {
      const userInfo = await invoke("plugin:browser|authenticate");

      log.debug("auth", userInfo);

      userInfoData.set(userInfo);

      return;
    }

    const awsCognitoTokenCache = createAwsCognitoTokenCache();

    awsCognitoTokenCache.getAuthorizationCodeInteractive();
  }
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
              <div
                class="menu-dropdown-item"
                on:click={() => onMenuItemClick(menu.title)}
              >
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
  <div class="actions">
    <div
      class="authenticate"
      class:cursor-pointer={!$userInfoData}
      title={$userInfoData
        ? `Welcome back ${$userInfoData.name}`
        : "Authenticate"}
      on:click={authenticate}
    >
      {userInitials}
    </div>
  </div>
</div>

<style lang="postcss">
  .root {
    @apply h-8 flex flex-row justify-between items-center flex-1 whitespace-nowrap;
  }

  .menus {
    @apply flex flex-row h-full flex-1 space-x-1 justify-center md:justify-start;
  }

  .brand {
    @apply flex items-center italic px-2;
  }

  .menu {
    @apply hidden sm:flex items-center cursor-pointer z-20 hover:bg-gray-500;
  }

  .menu-title {
    @apply px-2;
  }

  .menu-dropdown {
    @apply absolute top-7 rounded-b-sm;
  }

  .menu-dropdown-items {
    @apply bg-gray-800 py-1;
  }

  .menu-dropdown-item {
    @apply hover:bg-gray-500 cursor-pointer px-6 py-0.5;
  }

  .document-title {
    @apply hidden sm:flex;
  }

  .actions {
    @apply justify-end hidden sm:flex items-center flex-1 pr-2;
  }

  .authenticate {
    @apply flex justify-center items-center rounded-full bg-orange-700 bg-opacity-80 h-6 w-6 text-xs text-white font-bold;
  }
</style>
