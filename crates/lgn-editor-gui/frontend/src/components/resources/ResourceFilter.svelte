<script lang="ts">
  import Icon from "@iconify/svelte";
  import { createEventDispatcher } from "svelte";

  import Button from "@lgn/web-client/src/components/Button.svelte";

  import TextInput from "@/components/inputs/TextInput.svelte";

  const dispatch = createEventDispatcher<{ filter: { name: string } }>();

  let name = "";

  function resetname() {
    name = "";

    dispatch("filter", { name });
  }

  function submit(event: Event /* SubmitEvent */) {
    event.preventDefault();

    dispatch("filter", { name });
  }
</script>

<form class="root" on:submit={submit}>
  <div class="flex-grow">
    <TextInput
      bind:value={name}
      size="default"
      fluid
      placeholder="Resource Name"
    >
      <div class="clear" slot="rightExtension" on:click={resetname}>
        <Icon icon="ic:baseline-close" title="Reset filter" />
      </div>
    </TextInput>
  </div>
  <div class="flex-shrink">
    <Button type="submit">Search</Button>
  </div>
</form>

<style lang="postcss">
  .root {
    @apply flex items-center h-10 w-full space-x-1 justify-end py-1 px-2;
  }

  .clear {
    @apply flex justify-center items-center h-full w-full cursor-pointer;
  }
</style>
