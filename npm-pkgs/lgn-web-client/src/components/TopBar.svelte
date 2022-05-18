<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";

  import { authClient } from "../lib/auth";
  import userInfo from "../orchestrators/userInfo";
  import type { DevSettingsValue } from "../stores/devSettings";
  import BrandLogo from "./BrandLogo.svelte";
  import MenuBar from "./menu/MenuBar.svelte";
  import type { MenuItemDescription } from "./menu/lib/MenuItemDescription";

  const { data: userInfoData } = userInfo;

  export let documentTitle: string | null = null;

  export let devSettings: DevSettingsValue | null = null;

  export let mainMenuItemDescriptions: MenuItemDescription[];

  let topBarHandle: HTMLDivElement | undefined;

  let topBarMinimize: HTMLDivElement | undefined;

  let topBarMaximize: HTMLDivElement | undefined;

  let topBarClose: HTMLDivElement | undefined;

  let devSettingsTitle: string | null = null;

  onMount(() => {
    if (!window.isElectron || !window.electron) {
      return;
    }

    const minimize = window.electron.minimizeMainWindow;
    const toggleMaximize = window.electron.toggleMaximizeMainWindow;
    const close = window.electron.closeMainWindow;

    function topBarMouseDownListener(event: MouseEvent) {
      event.detail === 1 ? event.preventDefault() : toggleMaximize();
    }

    topBarHandle?.addEventListener("mousedown", topBarMouseDownListener);
    topBarMinimize?.addEventListener("click", minimize);
    topBarMaximize?.addEventListener("click", toggleMaximize);
    topBarClose?.addEventListener("click", close);

    return () => {
      topBarHandle?.removeEventListener("mousedown", topBarMouseDownListener);
      topBarMinimize?.removeEventListener("click", minimize);
      topBarMaximize?.removeEventListener("click", toggleMaximize);
      topBarClose?.removeEventListener("click", close);
    };
  });

  $: userInitials =
    $userInfoData && $userInfoData.given_name && $userInfoData.family_name
      ? `${$userInfoData.given_name[0]}${$userInfoData.family_name[0]}`
      : null;

  async function authenticate() {
    if (window.isElectron) {
      // TODO: When the application is running on Electron
      // it should use the node native module
    }

    const authorizationUrl = await authClient.getAuthorizationUrl();

    if (authorizationUrl) {
      window.location.href = authorizationUrl;
    }
  }

  $: if (devSettings) {
    devSettingsTitle = `Editor server url: ${devSettings.editorServerUrl}\nRuntime server url: ${devSettings.runtimeServerUrl}`;
  }
</script>

<div class="root" class:electron={window.isElectron}>
  <div class="menus">
    <div class="brand" title="Legion Editor">
      <BrandLogo class="brand-logo" />
    </div>
    <MenuBar items={mainMenuItemDescriptions} />
  </div>
  <div class="handle" bind:this={topBarHandle} style="-webkit-app-region: drag">
    <div class="document-title">
      {#if documentTitle}
        {documentTitle}
      {:else}
        Legion Sample Project
      {/if}
    </div>
  </div>
  <div class="actions">
    {#if devSettings}
      <div class="dev-settings" title={devSettingsTitle}>
        <Icon
          class="w-full h-full"
          icon="ic:baseline-settings"
          title={devSettingsTitle}
        />
      </div>
    {/if}
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
        <Icon icon="ic:baseline-account-circle" />
      {/if}
    </div>
    {#if window.isElectron}
      <div class="window-decorations">
        <div class="window-decoration" bind:this={topBarMinimize}>
          <Icon icon="ic:baseline-minimize" />
        </div>
        <div class="window-decoration" bind:this={topBarMaximize}>
          <Icon icon="ic:sharp-crop-square" />
        </div>
        <div class="window-decoration danger" bind:this={topBarClose}>
          <Icon icon="ic:baseline-close" />
        </div>
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  .root {
    @apply h-8 flex flex-row justify-between items-center flex-1 whitespace-nowrap;
  }

  .root.electron {
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

  .handle {
    @apply flex flex-row flex-1 flex-grow flex-shrink-0 justify-center;
  }

  .document-title {
    @apply hidden sm:flex;
  }

  .actions {
    @apply flex flex-row h-full justify-end items-center space-x-4;
  }

  .dev-settings {
    @apply h-6 w-6;
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
