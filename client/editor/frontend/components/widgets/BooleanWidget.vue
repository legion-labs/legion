<template>
  <span>
    <v-switch dense :disabled="readonly" v-model="localValue"></v-switch>
  </span>
</template>

<style scoped>
span {
  display: inline-block;
  margin: auto;
}
</style>

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