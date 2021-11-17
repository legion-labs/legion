<script>
import { u32ToHexcolor, hexcolorToU32 } from "~/modules/conversion";

export default {
  name: "ColorEditor",
  // eslint-disable-next-line vue/require-prop-types
  props: ["value"],
  computed: {
    localValue: {
      get() {
        return u32ToHexcolor(this.value);
      },
      set(val) {
        this.$emit("input", hexcolorToU32(val));
      },
    },
  },
};
</script>

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
