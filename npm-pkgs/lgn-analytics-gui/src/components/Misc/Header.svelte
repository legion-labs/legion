<script lang="ts">
  import { getContext, onMount } from "svelte";
  import { link } from "svelte-navigator";

  import { authClient } from "@lgn/web-client/src/lib/auth";
  import type { ThemeStore } from "@lgn/web-client/src/stores/theme";

  import { themeContextKey } from "@/constants";

  import iconPath from "../../../icons/128x128.png";
  import User from "../List/User.svelte";

  let user: string | undefined;

  const theme = getContext<ThemeStore>(themeContextKey);

  // TODO: Drop this whole logic when the dark theme is mature enough
  // Feel free to set this const to "true" to enable fast theme switching
  const themeIsTogglable = false;

  onMount(async () => {
    user = (await authClient.userInfo()).name;
  });

  function toggleTheme(event: MouseEvent) {
    if (event.ctrlKey && event.shiftKey) {
      event.preventDefault();

      $theme.name = $theme.name === "dark" ? "light" : "dark";
    }
  }
</script>

<div class="w-full flex justify-between pl-6 pt-4 pr-4">
  <div class="flex items-center gap-3">
    <a href="/" use:link>
      <img
        src={iconPath}
        alt="logo"
        style="height:24px"
        class="inline"
        on:click={themeIsTogglable ? toggleTheme : undefined}
      />
      <span class="font-bold text-xl text-content-87">
        <a href="/" use:link>Legion Performance Analytics</a>
      </span>
    </a>
  </div>
  {#if user}
    <div class="flex justify-between items-center">
      <User {user} />
    </div>
  {/if}
</div>
