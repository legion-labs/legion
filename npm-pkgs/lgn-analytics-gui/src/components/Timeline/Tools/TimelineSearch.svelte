<script lang="ts">
  import { getContext } from "svelte";

  import { l10nOrchestratorContextKey } from "@lgn/web-client/src/constants";

  import { TimelineContext } from "../Stores/TimelineContext";

  const searchStore = TimelineContext.search;

  const { t } = getContext(l10nOrchestratorContextKey);
  export let searching: boolean;
</script>

<div class="flex flex-row h-full w-full justify-end space-x-1">
  <input
    type="text"
    class="h-8 w-60 placeholder rounded-l-sm pl-2 surface"
    placeholder={$t("timeline-search")}
    on:focus={() => (searching = true)}
    on:blur={() => (searching = false)}
    bind:value={$searchStore}
  />
  <button
    class="text-sm surface h-8 w-8 rounded-r-sm text-opacity-60"
    on:click={() => ($searchStore = "")}
  >
    <i class="bi bi-x-circle" />
  </button>
</div>
