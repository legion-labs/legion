<script lang="ts">
  import { getL10nOrchestratorContext } from "@/contexts";

  import { TimelineContext } from "../Stores/TimelineContext";

  const searchStore = TimelineContext.search;

  const { t } = getL10nOrchestratorContext();

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
