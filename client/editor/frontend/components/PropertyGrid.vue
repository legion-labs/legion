<template>
  <div class="d-flex flex-column">
    <ResourceSelector></ResourceSelector>
    <v-color-picker
      id="color-picker"
      :value="color"
      @input="onColorChange"
      mode="rgba"
    ></v-color-picker>
    <v-slider
      class="speed-selector"
      min="-10"
      max="10"
      :value="speed"
      @input="onSpeedChange"
    >
      <template v-slot:append>
        <v-text-field
          :value="speed"
          @input="onSpeedChange"
          class="mt-0 pt-0"
          hide-details
          single-line
          type="number"
          style="width: 40px"
        ></v-text-field>
      </template>
    </v-slider>
  </div>
</template>

<style>
#color-picker {
  margin: 1em 0;
}

.speed-selector {
  width: 100%;
}
</style>

<script>
export default {
  methods: {
    onColorChange(color) {
      this.$emit("color-change", color);
    },
    onSpeedChange(speed) {
      this.$emit("speed-change", speed);
    },
  },
  props: ["color", "speed"],
  data() {
    return {};
  },
};
</script>