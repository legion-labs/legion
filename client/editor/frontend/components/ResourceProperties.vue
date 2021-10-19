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
  props: ["resource"],
  watch: {
    resource: {
      immediate: true,
      handler(val) {
        this.queryResourceProperties(val.id);
      },
    },
  },
  methods: {
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
