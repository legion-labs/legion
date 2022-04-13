<script lang="ts">
  import type { Writable } from "svelte/store";

  type Value = $$Generic;

  // Copied from svelte-form/types as it couldn't be imported
  type Field<Value> = {
    name: string;
    value: Value;
    valid: boolean;
    invalid: boolean;
    dirty: boolean;
    errors: string[];
  };

  /** Basically an `width: 100%` style so that the parent can control the width */
  export let fluid = false;

  export let field: Writable<Field<Value>>;

  export let orientation: "horizontal" | "vertical" = "vertical";
</script>

<div class:w-full={fluid}>
  <!-- svelte-ignore a11y-label-has-associated-control -->
  <label class:w-full={fluid}>
    <div
      class="field"
      class:horizontal={orientation === "horizontal"}
      class:vertical={orientation === "vertical"}
      class:w-full={fluid}
    >
      {#if $$slots.label}
        <div><slot name="label" /></div>
      {/if}
      <div class:w-full={fluid}>
        <slot name="input" />
      </div>
      {#if $field.errors}
        {#each $field.errors as error (error)}
          <div class="text-red-700">
            <slot name="error" {error}>{error}</slot>
          </div>
        {/each}
      {/if}
    </div>
  </label>
</div>

<style lang="postcss">
  .field {
    @apply flex w-full;
  }

  .field.vertical {
    @apply flex-col space-y-1;
  }

  .field.horizontal {
    @apply flex-row space-x-2 items-center;
  }
</style>
