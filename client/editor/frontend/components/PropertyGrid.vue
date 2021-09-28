<template>
  <v-data-table
    :headers="headers"
    :items="items"
    :items-per-page="10"
    class="elevation-1"
  ></v-data-table>
</template>

<script>
export default {
  data() {
    return {
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