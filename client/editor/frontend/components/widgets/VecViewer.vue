<template>
  <pre>(<span v-for="(dimension, index) in localValue" :key="index"><span v-if="index > 0">, </span>{{ localValue[index] }}</span>)</pre>
</template>

<script>
export default {
  name: "Vec3Viewer",
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