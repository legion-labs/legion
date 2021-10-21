<template>
  <v-text-field
    type="number"
    dense
    filled
    hide-details
    single-line
    outlined
    v-model="localValue"
  ></v-text-field>
</template>

<script>
export default {
  name: "NumberEditor",
  props: {
    value: Number,
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