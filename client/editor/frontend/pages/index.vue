<template>
  <div class="d-flex flex-column flex-grow-1">
    <div class="d-flex flex-row flex-grow-1">
      <PropertyGrid
        :color="color"
        @color-change="onColorChange"
        :speed="speed"
        @speed-change="onSpeedChange"
        class="flex-shrink-1"
      />
      <Video :color="color" :speed="speed" class="flex-grow-1" />
    </div>
  </div>
</template>

<script>
export default {
  methods: {
    onColorChange(color) {
      this.color = color;
    },
    onSpeedChange(speed) {
      this.speed = speed;
    },
  },
  data() {
    return {
      color: "#FF0000FF",
      speed: 1.0,
    };
  },
};
</script>
