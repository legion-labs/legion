<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import { dropzone } from "@lgn/web-client/src/actions/dnd";

  import { createResourcePathId } from "@/lib/resourceBrowser";
  import { resourceEntries } from "@/orchestrators/resourceBrowserEntries";

  import TextInput from "../../inputs/TextInput.svelte";

  export let value: string;

  export let resource_type: string | null;

  export let readonly = false;

  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  async function onDrop({
    detail: { item: draggedEntry },
  }: CustomEvent<{ item: any; originalEvent: DragEvent }>) {
    if (resource_type) {
      let new_value = createResourcePathId(resource_type, draggedEntry);
      if (new_value) {
        value = new_value;
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
        let sub_value = value.slice(index + 1);
        index = sub_value.indexOf("|");
        if (index != -1) {
          result += "/" + sub_value.slice(undefined, index);
        }
      }
    }
    return result;
  }
</script>

<!-- For now the string property is only a TextInput but it might change -->
<div
  use:dropzone={{ accept: "RESOURCE" }}
  on:dnd-drop={onDrop}
  title={getName()}
>
  <TextInput on:input bind:value fluid autoSelect {readonly} />
</div>
