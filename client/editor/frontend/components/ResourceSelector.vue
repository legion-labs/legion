<template>
  <v-form>
    <v-autocomplete
      v-model="resource"
      :loading="loading"
      :items="resourceDescriptions"
      :search-input.sync="search"
      class="mx-4"
      flat
      label="Enter a resource path"
      return-object
      item-text="path"
      item-value="id"
    ></v-autocomplete>
  </v-form>
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
      val && this.querySelections(val);
    },
  },
  methods: {
    async querySelections(v) {
      this.loading = true;

      try {
        const resp = await search_resources();

        this.resourceDescriptions = resp.resource_descriptions;
      } finally {
        this.loading = false;
      }
    },
  },
  async mounted() {
    try {
      await this.querySelections("");
    } catch (e) {
      console.log("Failed to query initial resources: ", e);
    }

    if (this.resourceDescriptions.length > 0) {
      this.resource = this.resourceDescriptions[0];
    }
  },
};
</script>
