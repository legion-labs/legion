<template>
  <div class="d-flex flex-column">
    <ResourceSelector v-model="resourceDescription"></ResourceSelector>
    <ResourceProperties
      v-if="resourceDescription"
      :resource-description="resourceDescription"
      @resource-change="$emit('resource-change', $event)"
    ></ResourceProperties>
  </div>
</template>

<script>
export default {
  data() {
    return {
      resourceDescription: null,
    };
  },
};
</script>