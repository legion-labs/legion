<template>
  <v-card dark color="blue lighten-1">
    <v-card-title class="text-h5 blue">Resource selection</v-card-title>
    <v-card-text>Select a resource to edit.</v-card-text>
    <v-card-text>
      <v-autocomplete
        v-model="resource"
        :loading="loading"
        :items="resourceDescriptions"
        :search-input.sync="search"
        class="mx-4"
        flat
        label="Enter a resource path"
        solo-inverted
        return-object
        item-text="path"
        item-value="id"
      ></v-autocomplete>
    </v-card-text>
  </v-card>
</template>

<script>
import { search_resources } from "~/modules/api";

export default {
  name: "ResourceSelector",
  data() {
    return {
      loading: false,
      resourceDescriptions: [],
      search: null,
      resource: null,
    };
  },
  watch: {
    resource(val) {
      this.$emit("input", val);
    },
    search(val) {
      val && val !== this.select && this.querySelections(val);
    },
  },
  methods: {
    querySelections(v) {
      this.loading = true;

      search_resources().then((resp) => {
        this.resourceDescriptions = resp.resource_descriptions;
        this.loading = false;
      });
    },
  },
  mounted() {},
};
</script>
