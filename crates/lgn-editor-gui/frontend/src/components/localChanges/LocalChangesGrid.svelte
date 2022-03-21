<script lang="ts">
  import { fileName } from "@/lib/path";
  import { StagedResource_ChangeType as ChangeType } from "@lgn/proto-editor/dist/source_control";
  import type { StagedResource } from "@lgn/proto-editor/dist/source_control";
  import FileIcon from "../FileIcon.svelte";

  export let stagedResources: StagedResource[];
</script>

<div class="root">
  {#each stagedResources as resource, index (index)}
    <div
      class="resource"
      title={resource.info?.path}
      class:border-green-600={resource.changeType === ChangeType.Add}
      class:border-orange-400={resource.changeType === ChangeType.Edit}
      class:border-red-500={resource.changeType === ChangeType.Delete}
    >
      <div class="resource-card">
        <div class="resource-icon">
          <FileIcon
            class="h-20 w-20 text-white text-opacity-60"
            textClass="text-gray-800"
            text={resource.info?.type ?? "unknown"}
          />
        </div>
        <div
          class="resource-text"
          class:bg-green-600={resource.changeType === ChangeType.Add}
          class:bg-orange-400={resource.changeType === ChangeType.Edit}
          class:bg-red-500={resource.changeType === ChangeType.Delete}
          title={resource.info?.path}
        >
          <div class="truncate">{fileName(resource.info?.path || "")}</div>
        </div>
      </div>
    </div>
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 xl:grid-cols-8 gap-4;
  }

  .resource {
    @apply flex items-center justify-center border;
  }

  .resource-card {
    @apply w-full bg-gray-800 shadow-xl rounded-sm;
  }

  .resource-icon {
    @apply flex flex-col items-center p-4;
  }

  .resource-text {
    @apply flex items-center h-12 w-full px-2 bg-opacity-60 rounded-b-sm;
  }
</style>
