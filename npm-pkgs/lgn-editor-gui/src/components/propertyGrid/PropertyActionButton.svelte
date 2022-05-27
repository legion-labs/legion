<script lang="ts">
  import Icon from "@iconify/svelte";

  export let icon: string;
</script>

<div class="action-button" on:click>
  <Icon {icon} />
</div>

<style>
  div {
    @apply h-6 w-6 bg-surface-500 flex justify-center items-center cursor-pointer border-[1px] border-black;
  }
</style>
