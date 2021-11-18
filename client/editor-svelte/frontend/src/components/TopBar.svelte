<script lang="ts">
  export let documentTitle: string | null = null;

  const menuItems = [
    { id: 1, title: "File" },
    { id: 2, title: "Edit" },
    { id: 3, title: "Layer" },
    { id: 4, title: "Document" },
    { id: 5, title: "View" },
    { id: 6, title: "Help" },
  ];
</script>

<div class="flex flex-row justify-between space-x-2">
  <div class="flex flex-row flex-1 h-7 space-x-1 text-sm">
    <div class="flex items-center italic px-2">Legion</div>
    {#each menuItems as menuItem}
      <div class="flex items-center px-2 hover:bg-gray-400 cursor-pointer">
        {menuItem.title}
      </div>
    {/each}
  </div>
  <div
    class="flex flex-row justify-center items-center flex-1 whitespace-nowrap"
  >
    {#if documentTitle}
      {documentTitle}
    {:else}
      Untitled document
    {/if}
  </div>
  <div class="flex-1" />
</div>
