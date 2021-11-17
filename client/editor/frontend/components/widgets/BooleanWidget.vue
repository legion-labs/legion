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

<template>
  <span>
    <v-switch v-model="localValue" dense :disabled="readonly"></v-switch>
  </span>
</template>

<style scoped>
span {
  display: inline-block;
  margin: auto;
}
</style>
