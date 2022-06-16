<script lang="ts">
  import type { Writable } from "svelte/store";

  import type { SourceControl } from "@lgn/apis/editor";

  import contextMenu from "@/actions/contextMenu";
  import { fileName } from "@/lib/path";
  import { localChangesContextMenuId } from "@/stores/contextMenu";

  import FileIcon from "../FileIcon.svelte";

  export let stagedResources: SourceControl.StagedResource[];
  export let selectedResource: Writable<SourceControl.StagedResource | null>;
</script>

<div class="root">
  {#each stagedResources as resource, index (index)}
    <div
      class="resource"
      class:selected={$selectedResource === resource}
      title={resource.info?.path || "Unknown path"}
      on:click={() => ($selectedResource = resource)}
      on:mousedown={() => ($selectedResource = resource)}
      use:contextMenu={localChangesContextMenuId}
    >
      <div
        class="resource-icon"
        class:border-green-600={resource.change_type === "Add"}
        class:border-orange-400={resource.change_type === "Edit"}
        class:border-red-500={resource.change_type === "Delete"}
      >
        <FileIcon
          class="h-20 w-20 text-white text-opacity-60"
          textClass="text-gray-800"
          text={resource.info?.type ?? "unknown"}
        />
      </div>
      <div
        class="resource-text"
        class:bg-green-600={resource.change_type === "Add"}
        class:bg-orange-400={resource.change_type === "Edit"}
        class:bg-red-500={resource.change_type === "Delete"}
        title={resource.info?.path}
      >
        <div class="truncate">{fileName(resource.info?.path || "")}</div>
      </div>
    </div>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 xl:grid-cols-8 gap-4 px-4 pt-2 pb-4 overflow-y-auto h-full w-full;
  }

  .resource {
    @apply w-full shadow-xl rounded-sm h-40;
  }

  .resource.selected {
    @apply bg-black;
  }

  .resource-icon {
    @apply flex flex-col items-center p-4 bg-gray-700 border rounded-t-sm border-b-0;
  }

  .resource-text {
    @apply flex items-center h-12 w-full px-2 bg-opacity-60 rounded-b-sm;
  }
</style>
