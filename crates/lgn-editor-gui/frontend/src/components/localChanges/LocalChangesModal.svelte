<script lang="ts">
  import Icon from "@iconify/svelte";

  import Button from "@lgn/web-client/src/components/Button.svelte";
  import Modal from "@lgn/web-client/src/components/modal/Modal.svelte";

  import {
    stagedResources,
    stagedResourcesMode,
    submitToMain,
    syncFromMain,
  } from "@/stores/stagedResources";

  import TextArea from "../inputs/TextArea.svelte";
  import LocalChangesGrid from "./LocalChangesGrid.svelte";
  import LocalChangesList from "./LocalChangesList.svelte";

  export let close: () => void;

  let commitMessage = "";

  let loading = false;

  async function submit() {
    loading = true;

    await submitToMain(commitMessage);

    loading = false;

    close();
  }
</script>

<form class="root" on:submit|preventDefault={submit}>
  <Modal on:close={close} size="lg">
    <div slot="title">
      <div>Local Changes</div>
    </div>
    <div class="body" slot="body">
      <div class="sync-button">
        <Button fluid size="lg" variant="success" on:click={syncFromMain}>
          Sync from main
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
      <div class="local-changes">
        {#if $stagedResources && $stagedResources.length}
          {#if $stagedResourcesMode === "card"}
            <LocalChangesGrid stagedResources={$stagedResources} />
          {:else if $stagedResourcesMode === "list"}
            <LocalChangesList stagedResources={$stagedResources} />
          {/if}
        {:else}
          <div class="no-local-changes">
            <em>No local changes</em>
          </div>
        {/if}
      </div>
      <div class="commit-message">
        <TextArea
          bind:value={commitMessage}
          fluid
          placeholder="Commit Message"
        />
      </div>
    </div>
    <div class="footer" slot="footer">
      <div class="buttons">
        <div>
          <Button size="lg" on:click={close} disabled={loading}>Cancel</Button>
        </div>
        <div>
          <Button variant="success" size="lg" type="submit" disabled={loading}>
            Submit
          </Button>
        </div>
      </div>
    </div>
  </Modal>
</form>

<style lang="postcss">
  .root {
    @apply flex items-center justify-center h-full w-full;
  }

  .body {
    @apply flex flex-col flex-1 py-4 space-y-2;
  }

  .body .sync-button {
    @apply px-4 flex-shrink-0;
  }

  .body .mode {
    @apply flex flex-row px-4 justify-end space-x-1 flex-shrink-0;
  }

  .body .local-changes {
    @apply flex-1 overflow-y-auto;
  }

  .body .commit-message {
    @apply px-4 flex-shrink-0;
  }

  .body .no-local-changes {
    @apply flex justify-center items-center h-full w-full text-xl font-bold;
  }

  .footer {
    @apply flex flex-row justify-end w-full;
  }

  .footer .buttons {
    @apply flex flex-row space-x-2;
  }
</style>
