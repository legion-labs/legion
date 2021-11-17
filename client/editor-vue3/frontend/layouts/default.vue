<script lang="ts" setup>
import TopBar from "~/components/TopBar.vue";
</script>

<template>
  <div class="h-screen w-full">
    <TopBar />
    <div class="h-[calc(100vh-1.75rem)] w-full">
      <slot />
    </div>
  </div>
</template>
