<script lang="ts">
  import modal from "@/stores/modal";
  import {
    stagedResources,
    syncFromMain,
    stagedResourcesMode,
  } from "@/stores/stagedResources";
  import Icon from "@iconify/svelte";
  import Button from "@lgn/web-client/src/components/Button.svelte";
  import LocalChangesModal from "./LocalChangesModal.svelte";

  function openLocalChangesModal() {
    if (!$stagedResources || !$stagedResources.length) {
      return;
    }

    modal.open(Symbol.for("local-changes"), LocalChangesModal);
  }
</script>

<div class="root">
  <div class="flex flex-row space-x-2">
    <Button on:click={syncFromMain}>Sync from main</Button>
    <Button
      disabled={!$stagedResources || !$stagedResources.length}
      on:click={openLocalChangesModal}
    >
      Submit to main
    </Button>
  </div>
  <div class="mode">
    <div>
      <Button
        variant={$stagedResourcesMode === "card" ? "active" : "notice"}
        on:click={() => ($stagedResourcesMode = "card")}
        title="Card"
      >
        <Icon icon="ic:round-grid-view" />
      </Button>
    </div>
    <div>
      <Button
        variant={$stagedResourcesMode === "list" ? "active" : "notice"}
        on:click={() => ($stagedResourcesMode = "list")}
        title="Lard"
      >
        <Icon icon="ic:baseline-format-list-bulleted" />
      </Button>
    </div>
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row items-center justify-between px-2 h-10 w-full;
  }

  .mode {
    @apply flex flex-row space-x-1;
  }
</style>
