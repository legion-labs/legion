<script lang="ts">
  import { getContext, onMount } from "svelte";
  import { link } from "svelte-navigator";

  import { authClient } from "@lgn/web-client/src/lib/auth";
  import type { L10nOrchestrator } from "@lgn/web-client/src/orchestrators/l10n";

  import { l10nOrchestratorContextKey } from "@/constants";

  import iconPath from "../../../icons/128x128.png";
  import User from "../Process/User.svelte";

  let user: string | undefined;

  const { locale } = getContext<L10nOrchestrator<Fluent>>(
    l10nOrchestratorContextKey
  );

  onMount(async () => {
    user = (await authClient.userInfo()).name;
  });

  function toggleLocale(event: MouseEvent) {
    if (event.ctrlKey && event.shiftKey) {
      event.preventDefault();

      $locale = $locale === "fr-CA" ? "en-US" : "fr-CA";
    }
  }
</script>

<div class="w-full flex justify-between pl-6 pt-4 pr-4">
  <div class="flex items-center gap-3">
    <a href="/" use:link on:click={toggleLocale}>
      <img src={iconPath} alt="logo" style="height:24px" class="inline" />
      <span class="font-bold text-xl headline">
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
