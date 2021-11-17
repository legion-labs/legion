<template>
  <v-text-field
    v-model="localValue"
    type="number"
    dense
    filled
    hide-details
    single-line
    outlined
  ></v-text-field>
</template>

<script>
export default {
  name: "NumberEditor",
  props: {
    // eslint-disable-next-line vue/require-default-prop
    value: Number,
  },
  computed: {
    localValue: {
      get() {
        return this.value;
      },
      set(val) {
        const numVal = parseFloat(val);
        if (!isNaN(numVal)) {
          this.$emit("input", numVal);
        }
      },
    },
  },
};
</script>
