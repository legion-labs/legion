<script lang="ts">
  import Icon from "@iconify/svelte";
  import { createEventDispatcher } from "svelte";
  import { writable } from "svelte/store";

  import { debounced } from "@lgn/web-client/src/lib/store";

  import TextInput from "@/components/inputs/TextInput.svelte";

  const dispatch = createEventDispatcher<{ filter: { name: string } }>();

  const name = writable("");

  /** Debounce filter values update, if null data are synced instantly, `null` by default */
  export let debouncedMs: number | null = null;

  $: debouncedName = debouncedMs === null ? name : debounced(name, debouncedMs);

  $: dispatch("filter", { name: $debouncedName });

  function resetname() {
    $name = "";
  }
</script>

<div class="root">
  <TextInput
    bind:value={$name}
    size="default"
    fluid
    placeholder="Resource Name"
  >
    <div class="clear" slot="rightExtension" on:click={resetname}>
      <Icon icon="ic:baseline-close" title="Reset filter" />
    </div>
  </TextInput>
</div>

<style lang="postcss">
  .root {
    @apply flex items-center h-10 w-full justify-end py-2 px-1;
  }

  .clear {
    @apply flex justify-center items-center h-full w-full cursor-pointer;
  }
</style>
