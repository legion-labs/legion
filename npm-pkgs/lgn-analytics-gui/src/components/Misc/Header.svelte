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

<div class="w-full flex justify-between pl-6 pt-4 pr-4">
  <div class="flex items-center gap-3">
    <a href="/" on:click={toggleLocale}>
      <img src={iconPath} alt="logo" style="height:24px" class="inline" />
      <span class="font-bold text-xl headline">
        <a href="/">Legion Performance Analytics</a>
      </span>
    </a>
  </div>
  {#if user}
    <div class="flex justify-between items-center">
      <User {user} />
    </div>
  {/if}
</div>
