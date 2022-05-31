<script lang="ts">
  import { getContext } from "svelte";

  import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";
  import { userInfo } from "@lgn/web-client/src/orchestrators/userInfo";

  import iconPath from "@/assets/icons/128x128.png";

  import User from "../Process/User.svelte";

  const { availableLocales, locale } = getContext(l10nOrchestratorContextKey);

  const en = "en-US";
  const fr = "fr-CA";

  function setLocale(newLocale: string) {
    if ($availableLocales.includes(newLocale)) {
      $locale = newLocale;
    }
  }

  $: user = $userInfo?.name;
</script>

<div class="header">
  <div class="flex items-center gap-3">
    <a href="/" class="flex flex-row items-center space-x-1">
      <img src={iconPath} alt="logo" style="height:24px" class="inline" />
      <span class="font-medium text-xl headline">
        <a href="/">Analytics</a>
      </span>
    </a>
  </div>
  {#if $$slots.default}
    <slot />
  {/if}
  <div class="flex space-x-2">
    <div class="uppercase flex space-x-1 text-sm">
      <div
        class:text-primary={$locale === en}
        class:cursor-pointer={$locale !== en}
        on:click={() => setLocale(en)}
      >
        en
      </div>
      <div>/</div>
      <div
        class:text-primary={$locale === fr}
        class:cursor-pointer={$locale !== fr}
        on:click={() => setLocale(fr)}
      >
        fr
      </div>
    </div>
    {#if user}
      <div class="flex justify-between items-center">
        <User {user} iconOnly />
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  .header {
    @apply on-surface h-14 w-full flex justify-between items-center px-4 border-b border-black;
  }
</style>
