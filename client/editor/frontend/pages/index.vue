<template>
  <div class="d-flex flex-column flex-grow-1">
    <div class="d-flex flex-row flex-grow-1">
      <PropertyGrid
        @resource-change="resource = $event"
        class="flex-shrink-1"
      />
      <Video :resource="resource" class="flex-grow-1" />
    </div>
  </div>
</template>

<script>
export default {
  data() {
    return {
      resource: null,
    };
  },
};
</script>
