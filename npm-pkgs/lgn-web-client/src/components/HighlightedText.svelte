<script lang="ts">
  export let text: string;

  export let pattern: RegExp | string;

  $: patternRegExp =
    pattern instanceof RegExp ? pattern : new RegExp(`(${pattern})`, "i");

  $: highlightedTextParts = text.split(patternRegExp);
</script>

<div>
  {#each highlightedTextParts as part, index}
    {#if index % 2 === 0}
      {part}
    {:else}
      <mark class="highlighted-text">{part}</mark>
    {/if}
  {/each}
</div>

<style lang="postcss">
  .highlighted-text {
    @apply bg-orange-700 text-black font-bold;
  }
</style>
