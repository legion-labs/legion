<script lang="ts" setup>
import Panel from "~/components/Panel.vue";
</script>

<template>
  <div class="flex flex-row h-full w-full">
    <div class="flex flex-col flex-shrink-0 w-[1200px]">
      <Panel>
        <template #header> Untitled Document </template>
      </Panel>
    </div>
    <div class="flex-shrink-0 w-1" />
    <div class="flex flex-col w-full">
      <Panel>
        <template #header> Properties </template>
      </Panel>
      <div class="flex-shrink-0 h-1" />
      <Panel>
        <template #header> Layer Tree </template>
      </Panel>
    </div>
  </div>
</template>
