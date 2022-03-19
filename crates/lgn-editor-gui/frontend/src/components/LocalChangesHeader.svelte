<script lang="ts">
  import { commitStagedResources, getAllResources, syncLatest } from "@/api";
  import allResources from "@/stores/allResources";
  import { stagedResources } from "@/stores/stagedResources";
  import Icon from "@iconify/svelte";
  import Button from "@lgn/web-client/src/components/Button.svelte";
  import log from "@lgn/web-client/src/lib/log";

  type Mode = "card" | "list";

  export let mode: Mode = "card";

  function setMode(newMode: Mode) {
    mode = newMode;
  }

  function syncFromMain() {
    syncLatest();

    return allResources.run(getAllResources);
  }

  function submitToMain() {
    log.debug(log.json`Committing the following resources ${$stagedResources}`);

    return commitStagedResources();
  }
</script>

<div class="root">
  <div class="flex flex-row space-x-2">
    <Button on:click={syncFromMain}>Sync from main</Button>
    <Button
      disabled={!$stagedResources || !$stagedResources.length}
      on:click={submitToMain}
    >
      Submit to main
    </Button>
  </div>
  <div class="flex flex-row space-x-1">
    <Button
      variant={mode === "card" ? "active" : "notice"}
      on:click={() => setMode("card")}
      title="Card"
    >
      <Icon icon="ic:round-grid-view" />
    </Button>

    <Button
      variant={mode === "list" ? "active" : "notice"}
      on:click={() => setMode("list")}
      title="Lard"
    >
      <Icon icon="ic:baseline-format-list-bulleted" />
    </Button>
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row items-center justify-between px-2 h-10 w-full;
  }
</style>
