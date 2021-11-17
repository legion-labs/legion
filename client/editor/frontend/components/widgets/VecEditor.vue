<template>
  <span class="d-flex">
    <v-text-field
      v-for="(dimension, index) in localValue"
      :key="index"
      v-model="localValue[index]"
      type="number"
      dense
      filled
      hide-details
      single-line
      outlined
      class="shrink"
    ></v-text-field>
  </span>
</template>

<script>
export default {
  name: "VecEditor",
  props: {
    // eslint-disable-next-line vue/require-default-prop
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
