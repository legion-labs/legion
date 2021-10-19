<template>
  <v-color-picker
    id="control"
    v-model="localValue"
    mode="rgba"
    light
  ></v-color-picker>
</template>

<style scoped>
#control {
  margin-top: 1em;
  width: 100%;
  border-radius: 8px;
}
</style>
<script>
export default {
  name: "ColorEditor",
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