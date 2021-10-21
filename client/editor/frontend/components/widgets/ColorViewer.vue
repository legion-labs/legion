<template>
  <div id="color" :style="bgColor" :tile="localValue"></div>
</template>

<style scoped>
#color {
  height: 1.5em;
  width: 1.5em;
  border-radius: 2em;
  display: inline-block;
  border: 1px solid silver;
  background-color: var(--bg-color);
}
</style>

<script>
export default {
  name: "ColorViewer",
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
    bgColor() {
      return {
        "--bg-color": this.value,
      };
    },
  },
};
</script>