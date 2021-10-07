<template>
  <div class="d-flex flex-column align-center">
    <color-picker
      id="color-picker"
      variant="persistent"
      :value="hue"
      @input="onColorChange"
    ></color-picker>
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
    onColorChange(hue) {
      this.$emit("hue-change", hue);
    },
    onSpeedChange(speed) {
      this.$emit("speed-change", speed);
    },
  },
  props: ["hue", "speed"],
  data() {
    return {};
  },
};
</script>