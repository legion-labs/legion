<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Select from "../../inputs/Select.svelte";

  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  type EnumOption = { value: string; item: string };

  export let value: EnumOption;

  export let options: EnumOption[];

  export let disabled = false;

  function onSelect({ detail: entry }: CustomEvent<"" | EnumOption>) {
    if (entry == "") {
      dispatch("input", "");
    } else {
      dispatch("input", entry.value);
    }
  }
</script>

<Select bind:value {options} {disabled} on:select={onSelect}>
  <div slot="option" let:option>{option.item}</div>
</Select>
