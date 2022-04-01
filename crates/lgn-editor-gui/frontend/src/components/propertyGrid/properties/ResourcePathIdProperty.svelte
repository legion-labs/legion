<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import { dropzone } from "@lgn/web-client/src/actions/dnd";

  import { resourceDragAndDropType } from "@/constants";
  import { createResourcePathId } from "@/lib/resourceBrowser";
  import { resourceEntries } from "@/orchestrators/resourceBrowserEntries";

  import TextInput from "../../inputs/TextInput.svelte";

  export let value: string;

  export let resourceType: string | null;

  export let readonly = false;

  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  function onDrop({
    detail: { item: draggedEntry },
  }: CustomEvent<{ item: any; originalEvent: DragEvent }>) {
    if (resourceType) {
      console.log("dropped ", draggedEntry);

      const newValue = createResourcePathId(resourceType, draggedEntry);

      if (newValue) {
        value = newValue;

        dispatch("input", value);
      }
    }
  }

  function getName(): string {
    const entry = $resourceEntries.find((entry) =>
      value.startsWith(entry.item.id)
    );

    let result = "";

    if (entry) {
      result = entry.name;

      let index = value.indexOf("_");

      if (index != -1) {
        const subValue = value.slice(index + 1);

        index = subValue.indexOf("|");

        if (index != -1) {
          result += "/" + subValue.slice(undefined, index);
        }
      }
    }
    return result;
  }
</script>

<div
  use:dropzone={{ accept: resourceDragAndDropType }}
  on:dnd-drop={onDrop}
  title={getName()}
>
  <TextInput on:input bind:value fluid autoSelect {readonly} />
</div>
