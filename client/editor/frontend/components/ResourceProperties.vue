<template>
  <v-data-table
    :headers="headers"
    :items="properties"
    :items-per-page="-1"
    item-key="name"
    sort-by="name"
    group-by="group"
    show-group-by
    hide-default-footer
    :footer-props="{
      disableItemsPerPage: true,
    }"
    class="elevation-1"
  >
    <template #[`item.value`]="props">
      <ResourcePropertyEditor
        v-model="props.item.value"
        :ptype="props.item.ptype"
        @input="updateResource()"
      ></ResourcePropertyEditor>
    </template>
    <template #[`item.default_value`]="props">
      <ResourcePropertyEditor
        v-model="props.item.default_value"
        :ptype="props.item.ptype"
        :readonly="true"
      ></ResourcePropertyEditor>
    </template>
  </v-data-table>
</template>

<script>
import { get_resource_properties } from "~/modules/api";

export default {
  name: "ResourceProperties",
  data() {
    return {
      loading: false,
      headers: [
        {
          text: "Category",
          align: "start",
          sortable: true,
          value: "group",
          groupable: true,
        },
        {
          text: "Property",
          align: "start",
          sortable: true,
          value: "name",
          groupable: false,
        },
        {
          text: "Type",
          value: "ptype",
          groupable: false,
        },
        {
          text: "Value",
          value: "value",
          groupable: false,
        },
        {
          text: "Default",
          value: "default_value",
          groupable: false,
        },
      ],
      properties: [],
    };
  },
  props: ["resource-description"],
  watch: {
    resourceDescription: {
      immediate: true,
      handler(val) {
        this.queryResourceProperties(val.id);
      },
    },
    properties: {
      handler(val) {
        this.updateResource();
      },
    },
  },
  methods: {
    updateResource(resource) {
      if (!resource) {
        resource = {
          description: this.resourceDescription,
          properties: this.properties,
        };
      }

      // TODO: Replace this with a real call.
      this.$emit("resource-change", resource);
    },
    queryResourceProperties(resourceId) {
      this.loading = true;

      get_resource_properties(resourceId).then((resp) => {
        this.properties = resp.properties;
        this.loading = false;
      });
    },
  },
  mounted() {},
};
</script>
