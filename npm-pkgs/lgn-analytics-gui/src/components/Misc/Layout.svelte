<script lang="ts">
  import Header from "@/components/Misc/Header.svelte";

  import SubHeader from "./SubHeader.svelte";
</script>

<div class="layout">
  <div class="header">
    <Header>
      <slot name="header" />
    </Header>
  </div>

  {#if $$slots["sub-header"]}
    <div class="sub-header">
      <SubHeader>
        <slot name="sub-header" />
      </SubHeader>
    </div>
  {/if}

  <slot class="overflow-auto" name="content" />
</div>

<style lang="postcss">
  .layout {
    @apply w-full flex flex-col;
  }

  .header {
    @apply z-40 h-14 flex-shrink-0 sticky top-0;
  }

  .sub-header {
    @apply z-40 h-10 flex-shrink-0 sticky top-14;
  }
</style>
