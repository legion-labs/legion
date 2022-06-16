<script lang="ts">
  import type { Writable } from "svelte/store";

  import type { SourceControl } from "@lgn/apis/editor";

  import contextMenu from "@/actions/contextMenu";
  import { fileName } from "@/lib/path";
  import { localChangesContextMenuId } from "@/stores/contextMenu";

  export let stagedResources: SourceControl.StagedResource[];
  export let selectedResource: Writable<SourceControl.StagedResource | null>;
</script>

<div class="root">
  <div class="header">
    <div class="header-column w-1/12">change</div>
    <div class="header-column w-2/12">type</div>
    <div class="header-column w-9/12">path</div>
  </div>
  <div class="body">
    {#each stagedResources as resource, index (index)}
      <div
        class="resource-row"
        title={resource.info?.path || "Unknown path"}
        class:selected={$selectedResource === resource}
        on:click={() => ($selectedResource = resource)}
        on:mousedown={() => ($selectedResource = resource)}
        use:contextMenu={localChangesContextMenuId}
      >
        <div class="w-1/12 flex flex-row justify-center">
          <div
            class="w-4 h-4"
            class:bg-green-600={resource.change_type === "Add"}
            class:bg-orange-400={resource.change_type === "Edit"}
            class:bg-red-500={resource.change_type === "Delete"}
            title={resource.change_type}
          />
        </div>
        <div class="w-2/12 flex flex-row justify-center">
          <div>{fileName(resource.info?.type || "unknown")}</div>
        </div>
        <div class="w-9/12">
          <div class="truncate">
            {fileName(resource.info?.path || "Unknown path")}
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col h-full w-full;
  }

  .header {
    @apply uppercase flex flex-row items-center flex-shrink-0 h-8 w-full;
  }

  .header-column {
    @apply flex justify-center flex-shrink-0;
  }

  .body {
    @apply flex flex-col h-full flex-grow-0 overflow-y-auto;
  }

  .resource-row {
    @apply flex flex-row w-full h-12 bg-gray-700 odd:bg-opacity-50 even:bg-opacity-30 px-2 items-center;
  }

  .resource-row.selected {
    @apply bg-gray-800;
  }
</style>
