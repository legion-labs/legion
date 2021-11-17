<script>
export default {
  data() {
    return {
      resource: null,
    };
  },
};
</script>

<template>
  <div class="d-flex flex-column flex-grow-1">
    <div class="d-flex flex-row flex-grow-1">
      <PropertyGrid
        class="flex-shrink-1"
        @resource-change="resource = $event"
      />
      <Video :resource="resource" class="flex-grow-1" />
    </div>
  </div>
</template>

<style scoped>
div {
  max-height: 100%;
}
</style>
