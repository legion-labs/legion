<template>
  <div class="d-flex flex-column align-center">
    <color-picker
      id="color-picker"
      variant="persistent"
      :value="value"
      @input="onColorChange"
    ></color-picker>
    <v-data-table
      :headers="headers"
      :items="items"
      :items-per-page="10"
      class="elevation-1"
    ></v-data-table>
  </div>
</template>

<style>
#color-picker {
  margin: 1em 0;
}
</style>

<script>
export default {
  methods: {
    onColorChange(hue) {
      this.$emit("input", hue);
    },
  },
  props: ["value"],
  data() {
    console.log("hue", this.value);
    return {
      color: {
        hue: this.value,
        saturation: 100,
        luminosity: 50,
        alpha: 1,
      },
      headers: [
        {
          text: "Property",
          align: "Start",
          sortable: true,
          value: "name",
        },
        {
          text: "Value",
          value: "value",
        },
      ],
      items: [
        {
          name: "Color",
          value: "red",
        },
        {
          name: "Speed",
          value: 12,
        },
        {
          name: "Shape",
          value: "triangle",
        },
      ],
    };
  },
};
</script>