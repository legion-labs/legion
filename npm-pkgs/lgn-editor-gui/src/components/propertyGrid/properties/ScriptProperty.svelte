<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import Button from "@lgn/web-client/src/components/Button.svelte";

  import { currentResource } from "@/orchestrators/currentResource";
  import type { RootContext } from "@/routes/index.svelte";
  import ScriptEditor from "@lgn/web-client/src/components/ScriptEditor.svelte";
  import { fileName } from "@/lib/path";
  import { getContext } from "svelte";
  import { writable } from "svelte/store";

  // export let name: string;
  // export let path: string;

  // $: id = `script-${$currentResource?.id || ""}-${path}`;

  // $: payloadId = `${id}-payload`;

  // $: {
  //   // As soon as a payload exists it means the property is in "write" mode,
  //   // at that point the value is not owned by the property itself but rather by the tab
  //   // so we need to source the value from the payload instead of the property
  //   const payload = $tabPayloads[payloadId];

  //   if (payload?.type === "script") {
  //     value = payload.value;
  //   }
  // }

  export let value: string;
  export let readonly = false;

  const dispatch = createEventDispatcher<{ input: string }>();
  const context = getContext<RootContext>("root");
  const scriptEditor = writable(value);

  $: dispatch("input", value);
  $: dispatch("input", $scriptEditor);

  function openTab() {
    const layout = context.getLayout();

    layout.addComponent(
      ScriptEditor.name,
      {
        state: {
          theme: "vs-dark",
          lang: "rust",
          readonly,
          value: scriptEditor,
        },
      },
      `Script Editor: ${fileName(
        $currentResource?.description.path ?? "undefined"
      )}`,
      $currentResource?.id
    );

    // $tabPayloads[payloadId] = {
    //   type: "script",
    //   lang: "rust",
    //   readonly,
    //   value,
    // };

    // workspace.pushTabToPanel(
    //   viewportPanelId,
    //   {
    //     id,
    //     type: "script",
    //     label: `Script - ${
    //       $currentResourceName || "unknown resource"
    //     } - ${name}`,
    //     disposable: true,
    //     payloadId,
    //   },
    //   { focus: true }
    // );
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
