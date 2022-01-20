<script lang="ts">
  import { appWindow } from "@tauri-apps/api/window";
  import topBarMenu, {
    Id as TopBarMenuId,
    menus as topBarMenus,
  } from "../stores/topBarMenu";
  import userInfo from "../stores/userInfo";
  import log from "../lib/log";
  import clickOutside from "../actions/clickOutside";
  import { startUserAuth } from "../lib/auth";
  import BrandLogo from "./BrandLogo.svelte";
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";

  const { data: userInfoData } = userInfo;

  export let documentTitle: string | null = null;

  let topBarHandle: HTMLDivElement | undefined;

  let topBarMinimize: HTMLDivElement | undefined;

  let topBarMaximize: HTMLDivElement | undefined;

  let topBarClose: HTMLDivElement | undefined;

  onMount(() => {
    if (!window.__TAURI__) {
      return;
    }

    topBarHandle?.addEventListener("mousedown", topBarMouseDownListener);
    topBarMinimize?.addEventListener("click", appWindow.minimize);
    topBarMaximize?.addEventListener("click", appWindow.toggleMaximize);
    topBarClose?.addEventListener("click", appWindow.close);

    return () => {
      topBarHandle?.removeEventListener("mousedown", topBarMouseDownListener);
      topBarMinimize?.removeEventListener("click", appWindow.minimize);
      topBarMaximize?.removeEventListener("click", appWindow.toggleMaximize);
      topBarClose?.removeEventListener("click", appWindow.close);
    };
  });

  $: userInitials =
    $userInfoData && $userInfoData.given_name && $userInfoData.family_name
      ? `${$userInfoData.given_name[0]}${$userInfoData.family_name[0]}`
      : null;

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

  function authenticate() {
    startUserAuth();
  }

  // Used only in Tauri
  function topBarMouseDownListener(event: MouseEvent) {
    event.detail === 2 ? appWindow.toggleMaximize() : appWindow.startDragging();
  }
</script>

<div class="root" class:tauri={window.__TAURI__}>
  <div use:clickOutside={closeMenu} class="menus">
    <div class="brand" title="Legion Editor">
      <BrandLogo class="brand-logo" />
    </div>
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
          class:tauri={window.__TAURI__}
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
  <div class="handle" bind:this={topBarHandle}>
    <div class="document-title">
      {#if documentTitle}
        {documentTitle}
      {:else}
        Untitled document
      {/if}
    </div>
  </div>
  <div class="actions">
    <div
      class="authentication"
      class:cursor-pointer={!$userInfoData}
      title={$userInfoData
        ? `Welcome back ${$userInfoData.name}`
        : "Authenticate"}
      on:click={$userInfoData ? null : authenticate}
    >
      {#if userInitials}
        {userInitials}
      {:else}
        <Icon icon="mdi:account-circle" />
      {/if}
    </div>
    {#if window.__TAURI__}
      <div class="window-decorations">
        <div class="window-decoration" bind:this={topBarMinimize}>
          <Icon icon="mdi:window-minimize" />
        </div>
        <div class="window-decoration" bind:this={topBarMaximize}>
          <Icon icon="mdi:window-maximize" />
        </div>
        <div class="window-decoration danger" bind:this={topBarClose}>
          <Icon icon="mdi:window-close" />
        </div>
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  .root {
    @apply h-8 flex flex-row justify-between items-center flex-1 whitespace-nowrap;
  }

  .root.tauri {
    @apply h-10;
  }

  .menus {
    @apply flex flex-row h-full space-x-1 justify-center md:justify-start;
  }

  .brand {
    @apply h-full flex items-center px-2;
  }

  .brand :global(.brand-logo) {
    @apply h-full;
  }

  .menu {
    @apply hidden sm:flex items-center cursor-pointer z-20 hover:bg-gray-500;
  }

  .menu-title {
    @apply px-2;
  }

  .menu-dropdown {
    @apply absolute top-7 rounded-b-sm shadow-xl;
  }

  .menu-dropdown.tauri {
    @apply top-9;
  }

  .menu-dropdown-items {
    @apply bg-gray-800 py-1 rounded-b-sm;
  }

  .menu-dropdown-item {
    @apply hover:bg-gray-500 cursor-pointer px-6 py-0.5;
  }

  .handle {
    @apply flex flex-row flex-1 flex-grow flex-shrink-0 justify-center;
  }

  .document-title {
    @apply hidden sm:flex;
  }

  .actions {
    @apply flex flex-row h-full justify-end items-center space-x-4;
  }

  .authentication {
    @apply flex justify-center items-center flex-shrink-0 rounded-full mr-2 bg-orange-700 bg-opacity-80 h-6 w-6 text-xs text-white font-bold;
  }

  .authentication :global(svg) {
    @apply text-lg;
  }

  .window-decorations {
    @apply flex flex-row h-full space-x-2;
  }

  .window-decoration {
    @apply flex flex-row justify-center text-white items-center h-full px-4 w-12 hover:bg-gray-500 cursor-pointer;
  }

  .window-decoration.danger {
    @apply hover:bg-red-600;
  }
</style>
