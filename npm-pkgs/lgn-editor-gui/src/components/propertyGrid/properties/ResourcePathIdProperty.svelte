<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
  import { dropzone } from "@lgn/web-client/src/actions/dnd";

  import { getResourceNameFromEntries } from "@/components/propertyGrid/lib/propertyGrid";
  import { resourceDragAndDropType } from "@/constants";
  import type { Entry } from "@/lib/hierarchyTree";
  import { createResourcePathId } from "@/lib/resourceBrowser";
  import { resourceEntries } from "@/orchestrators/resourceBrowserEntries";

  import TextInput from "../../inputs/TextInput.svelte";

  export let value: string;

  export let resourceType: string | null;

  export let readonly = false;

  $: name = getResourceNameFromEntries($resourceEntries, value);

  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  function onDrop({
    detail: { item: draggedEntry },
  }: CustomEvent<{
    item: Entry<ResourceDescription>;
    originalEvent: DragEvent;
  }>) {
    if (resourceType) {
      const newValue = createResourcePathId(resourceType, draggedEntry);

      if (newValue) {
        value = newValue;

        dispatch("input", value);
      }
    }
  }
</script>

<div
  use:dropzone={{ accept: resourceDragAndDropType }}
  on:dnd-drop={onDrop}
  title={name}
>
  <TextInput on:input bind:value fluid autoSelect {readonly} />
</div>

<style lang="postcss">
  div {
    @apply w-full;
  }
</style>
