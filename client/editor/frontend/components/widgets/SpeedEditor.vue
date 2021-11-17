<template>
  <v-slider v-model="localValue" class="speed-selector" min="-10" max="10">
    <template #append>
      <v-text-field
        v-model="localValue"
        class="mt-0 pt-0"
        hide-details
        single-line
        type="number"
        style="width: 40px"
      ></v-text-field>
    </template>
  </v-slider>
</template>

<script>
export default {
  name: "SpeedEditor",
  // eslint-disable-next-line vue/require-prop-types
  props: ["value"],
  computed: {
    localValue: {
      get() {
        return this.value;
      },
      set(val) {
        this.$emit("input", val);
      },
    },
  },
};
</script>
