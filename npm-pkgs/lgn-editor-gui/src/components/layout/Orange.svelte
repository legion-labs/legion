<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  let date: number = Date.now();
  let interval: ReturnType<typeof setInterval>;

  onMount(() => {
    interval = setInterval(() => (date = Date.now()), 1000);
  });

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

{#each Array(15) as _, i (i)}
  <div>{date} {date} {date} {date}</div>
{/each}

<style>
  div {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    background-color: orange;
  }
</style>
