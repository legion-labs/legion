<script lang="ts">
  import { userInfo } from "@lgn/web-client/src/orchestrators/userInfo";

  import iconPath from "@/assets/icons/128x128.png";
  import { getL10nOrchestratorContext } from "@/contexts";

  import User from "../Process/User.svelte";

  const { locale } = getL10nOrchestratorContext();

  function toggleLocale(event: MouseEvent) {
    if (event.ctrlKey && event.shiftKey) {
      event.preventDefault();

      $locale = $locale === "fr-CA" ? "en-US" : "fr-CA";
    }
  }

  $: user = $userInfo?.name;
</script>

<div class="header">
  <div class="flex items-center gap-3">
    <a
      href="/"
      on:click={toggleLocale}
      class="flex flex-row items-center space-x-1"
    >
      <img src={iconPath} alt="logo" style="height:24px" class="inline" />
      <span class="font-medium text-xl headline">
        <a href="/">Analytics</a>
      </span>
    </a>
  </div>
  {#if $$slots.default}
    <slot />
  {/if}
  {#if user}
    <div class="flex justify-between items-center">
      <User {user} />
    </div>
  {/if}
</div>

<style lang="postcss">
  .header {
    @apply on-surface h-14 w-full flex justify-between items-center px-4 border-b border-black;
  }
</style>
