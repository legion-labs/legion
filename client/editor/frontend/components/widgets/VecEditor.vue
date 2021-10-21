<template>
  <span class="d-flex">
    <v-text-field
      v-for="(dimension, index) in localValue"
      :key="index"
      type="number"
      dense
      filled
      hide-details
      single-line
      outlined
      v-model="localValue[index]"
      class="shrink"
    ></v-text-field>
  </span>
</template>

<script>
export default {
  name: "VecEditor",
  props: {
    value: Array,
  },
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