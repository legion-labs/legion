<template>
  <v-slider class="speed-selector" min="-10" max="10" v-model="localValue">
    <template v-slot:append>
      <v-text-field
        v-model="localValue"
        class="mt-0 pt-0"
        hide-details
        single-line
        type="number"
        style="width: 40px"
      ></v-text-field>
    </template>
  </v-slider>
</template>

<script>
export default {
  name: "SpeedEditor",
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