<template>
  <div id="color" :style="bgColor" :tile="localValue"></div>
</template>

<style scoped>
#color {
  min-height: 1em;
  display: inline-block;
  min-width: 4em;
  border: 1px solid silver;
  background-color: var(--bg-color);
  border-radius: 1em;
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