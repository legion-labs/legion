<script lang="ts">
  import Button from "@lgn/web-client/src/components/Button.svelte";
  import viewportOrchestrator from "@/stores/viewport";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{ input: string }>();

  const key = Symbol();

  export let name: string;

  export let value: string;

  // TODO: Handle readonly mode in scripts
  export let readonly = false;

  $: dispatch("input", value);

  function openViewport() {
    viewportOrchestrator.add(
      key,
      {
        type: "script",
        name: `Script - ${name}`,
        onChange(newValue: string) {
          value = newValue;
        },
        value,
        readonly,
        lang: "rune",
        removable: true,
      },
      { focus: true }
    );
  }
</script>

<div class="root">
  <Button fluid on:click={openViewport}>
    <i>Edit...</i>
  </Button>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row justify-end w-full cursor-pointer;
  }
</style>
