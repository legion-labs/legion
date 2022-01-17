<script lang="ts">
  import { extension } from "@/lib/path";

  import { createEventDispatcher } from "svelte";
  import Icon, { IconName } from "../Icon.svelte";
  import TextInput from "../inputs/TextInput.svelte";

  type Item = $$Generic;

  const dispatch = createEventDispatcher<{
    select: Item;
    nameChange: { item: Item; newName: string };
  }>();

  // TODO: Temporary extension to icon name map, should be dynamic
  const iconNames = {
    pdf: "pdf",
    jpg: "image",
    jpeg: "image",
    png: "image",
    zip: "archive",
    rar: "archive",
  } as Record<string, IconName>;

  export let name: string;

  export let item: Item;

  export let isActive: boolean;

  let mode: "view" | "edit" = "view";

  $: nameValue = name;

  $: nameExtension = extension(name);

  $: iconName =
    (nameExtension && iconNames[nameExtension]) || "unknown-file-type";

  function onClick(item: Item) {
    dispatch("select", item);
  }

  function onDblClick() {
    if (mode === "view") {
      mode = "edit";
    }
  }

  function renameFile(event: Event) {
    event.preventDefault();

    mode = "view";

    if (nameValue.trim().length) {
      dispatch("nameChange", { item, newName: nameValue.trim() });

      nameValue = name;
    }
  }

  function cancelEdition(event?: KeyboardEvent) {
    if (event && event.key !== "Escape") {
      return;
    }

    mode = "view";

    nameValue = name;
  }

  $: if (!isActive) {
    cancelEdition();
  }
</script>

<div
  class="root"
  class:active-view={isActive && mode === "view"}
  class:lg-space={mode === "view"}
  on:click={() => onClick(item)}
  on:dblclick={onDblClick}
>
  {#if iconName}
    <div class="icon">
      <Icon name={iconName} />
    </div>
  {/if}
  {#if mode === "view"}
    <div>{name}</div>
  {:else if mode === "edit"}
    <form on:submit={renameFile} on:keydown={cancelEdition}>
      <TextInput autoFocus autoSelect bind:value={nameValue} />
    </form>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex items-center h-9 space-x-0.5 py-0.5 cursor-pointer border border-transparent;
  }

  .root.lg-space {
    @apply space-x-2;
  }

  .root.active-view {
    @apply border border-dotted border-orange-700 bg-orange-700 bg-opacity-10;
  }

  .icon {
    @apply flex items-center text-orange-700;
  }
</style>
