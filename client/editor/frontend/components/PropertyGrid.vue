<template>
  <div id="property-grid" :style="minWidth" class="d-flex flex-column">
    <v-subheader>Properties</v-subheader>
    <ResourceSelector v-model="resourceDescription"></ResourceSelector>
    <v-divider></v-divider>
    <ResourceProperties
      v-if="resourceDescription"
      class="flex-grow-1"
      :resource-description="resourceDescription"
      @resource-change="$emit('resource-change', $event)"
    ></ResourceProperties>
  </div>
</template>

<style scoped>
#property-grid {
  min-width: var(--min-width);
}
</style>

<script>
export default {
  data() {
    return {
      resourceDescription: null,
    };
  },
  computed: {
    minWidth() {
      return {
        "--min-width": Math.min(this.$vuetify.breakpoint.width / 5, 350) + "px",
      };
    },
  },
};
</script>