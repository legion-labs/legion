<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import Button from "@lgn/web-client/src/components/Button.svelte";

  import {
    currentResource,
    currentResourceName,
  } from "@/orchestrators/currentResource";
  import tabPayloads from "@/stores/tabPayloads";
  import workspace, { viewportPanelId } from "@/stores/workspace";

  const dispatch = createEventDispatcher<{ input: string }>();

  export let name: string;

  export let path: string;

  export let value: string;

  export let readonly = false;

  $: id = `script-${$currentResource?.id || ""}-${path}`;

  $: payloadId = `${id}-payload`;

  $: {
    // As soon as a payload exists it means the property is in "write" mode,
    // at that point the value is not owned by the property itself but rather by the tab
    // so we need to source the value from the payload instead of the property
    const payload = $tabPayloads[payloadId];

    if (payload?.type === "script") {
      value = payload.value;
    }
  }

  $: dispatch("input", value);

  function openTab() {
    $tabPayloads[payloadId] = {
      type: "script",
      lang: "rust",
      readonly,
      value,
    };

    workspace.pushTabToPanel(
      viewportPanelId,
      {
        id,
        type: "script",
        label: `Script - ${
          $currentResourceName || "unknown resource"
        } - ${name}`,
        disposable: true,
        payloadId,
      },
      { focus: true }
    );
  }
</script>

<div class="root">
  <Button fluid on:click={openTab}>
    <i>Edit...</i>
  </Button>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row justify-end w-full cursor-pointer;
  }
</style>
