<script lang="ts">
  import { StagedResource_ChangeType as ChangeType } from "@lgn/proto-editor/dist/source_control";
  import type { StagedResource } from "@lgn/proto-editor/dist/source_control";

  import { fileName } from "@/lib/path";

  export let stagedResources: StagedResource[];

  function changeTypeLabel(changeType: ChangeType) {
    switch (changeType) {
      case ChangeType.Add: {
        return "Add";
      }

      case ChangeType.Delete: {
        return "Delete";
      }

      case ChangeType.Edit: {
        return "Edit";
      }
    }
  }
</script>

<div class="root">
  <div class="header">
    <div class="header-column w-1/12">change</div>
    <div class="header-column w-2/12">type</div>
    <div class="header-column w-9/12">path</div>
  </div>
  <div class="body">
    {#each stagedResources as resource, index (index)}
      <div class="resource-row" title={resource.info?.path || "Unknown path"}>
        <div class="w-1/12 flex flex-row justify-center">
          <div
            class="w-4 h-4"
            class:bg-green-600={resource.changeType === ChangeType.Add}
            class:bg-orange-400={resource.changeType === ChangeType.Edit}
            class:bg-red-500={resource.changeType === ChangeType.Delete}
            title={changeTypeLabel(resource.changeType)}
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
    @apply flex flex-row w-full h-12 bg-gray-800 odd:bg-opacity-50 even:bg-opacity-30 px-2 items-center;
  }
</style>
