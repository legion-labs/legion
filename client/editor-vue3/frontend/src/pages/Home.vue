<script lang="ts" setup>
import Panel from "@/components/Panel.vue";
import TopBar from "@/components/TopBar.vue";
</script>

<template>
  <div class="h-screen w-full">
    <TopBar />
    <div class="h-[calc(100vh-1.75rem)] w-full">
      <div class="flex flex-row h-full w-full">
        <div class="flex flex-col flex-shrink-0 w-3/4">
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
    </div>
  </div>
</template>
