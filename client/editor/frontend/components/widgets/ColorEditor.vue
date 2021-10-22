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
import { u32_to_hexcolor, hexcolor_to_u32 } from "~/modules/conversion";

export default {
  name: "ColorEditor",
  props: ["value"],
  computed: {
    localValue: {
      get() {
        return u32_to_hexcolor(this.value);
      },
      set(val) {
        this.$emit("input", hexcolor_to_u32(val));
      },
    },
  },
};
</script>