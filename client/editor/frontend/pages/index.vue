<template>
  <div class="d-flex flex-column flex-grow-1">
    <div class="d-flex flex-row flex-grow-1">
      <PropertyGrid
        :hue="hue"
        @hue-change="onHueChange"
        :speed="speed"
        @speed-change="onSpeedChange"
        class="flex-shrink-1"
      />
      <Video :hue="hue" :speed="speed" class="flex-grow-1" />
    </div>
  </div>
</template>

<script>
export default {
  methods: {
    onHueChange(hue) {
      this.hue = hue;
    },
    onSpeedChange(speed) {
      this.speed = speed;
    },
  },
  data() {
    return {
      hue: 0,
      speed: 1.0,
    };
  },
};
</script>
