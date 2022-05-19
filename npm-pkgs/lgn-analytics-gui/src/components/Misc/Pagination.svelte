<script lang="ts">
  import { getL10nOrchestratorContext } from "@/contexts";

  const { t } = getL10nOrchestratorContext();

  export let begin: number;

  export let end: number;

  export let totalEntries: number;

  export let entriesPerPage: number;

  export let buildHref: (begin: number, end: number) => string;

  $: totalPage = Math.ceil(totalEntries / entriesPerPage);

  $: currentPage = Math.ceil(begin / entriesPerPage) + 1;
</script>

<div class="pagination">
  {#if begin > 0}
    <div class="link-container">
      <a
        class="link"
        href={buildHref(0, Math.min(entriesPerPage, totalEntries))}
        title={$t("global-pagination-first")}
      >
        <i class="bi-chevron-bar-left" />
      </a>
    </div>
    <div class="link-container">
      <a
        class="link"
        href={buildHref(Math.max(0, begin - entriesPerPage), begin)}
        title={$t("global-pagination-previous")}
      >
        <i class="bi-chevron-left" />
      </a>
    </div>
  {/if}
  <div class="page-counter">
    {currentPage} / {totalPage}
  </div>
  {#if end < totalEntries}
    <div class="link-container">
      <a
        class="link"
        href={buildHref(end, end + entriesPerPage)}
        title={$t("global-pagination-next")}
      >
        <i class="bi-chevron-right" />
      </a>
    </div>
    <div class="link-container">
      <a
        class="link"
        href={buildHref(totalEntries - entriesPerPage, totalEntries)}
        title={$t("global-pagination-last")}
      >
        <i class="bi-chevron-bar-right" />
      </a>
    </div>
  {/if}
</div>

<style lang="postcss">
  .pagination {
    @apply flex h-full;
  }

  .page-counter {
    @apply flex items-center justify-center h-10 px-2 headline text-xs bg-default border-b border-l border-[#3d3d3d];
  }

  .link-container {
    /* TODO: Use proper color */
    @apply text hover:headline h-10 w-10 border-l border-[#3d3d3d];
  }

  .link {
    @apply flex items-center justify-center h-full w-full;
  }
</style>
