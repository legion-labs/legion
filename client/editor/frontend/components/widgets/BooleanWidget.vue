<template>
  <v-switch flat :disabled="readonly" v-model="localValue"></v-switch>
</template>

<script>
export default {
  name: "BooleanWidget",
  props: {
    value: Boolean,
    readonly: {
      type: Boolean,
      default: false,
    },
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