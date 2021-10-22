<template>
  <v-data-table
    id="resource-properties"
    :headers="headers"
    :items="properties"
    :items-per-page="-1"
    item-key="name"
    sort-by="name"
    group-by="group"
    show-group-by
    hide-default-footer
    show-expand
    :footer-props="{
      disableItemsPerPage: true,
    }"
    class="elevation-1"
  >
    <template #[`item.name`]="{ item }">
      <div class="d-flex">
        <pre
          :class="{ changed: !isSetToDefault(item) }"
          :title="'Property of type ' + item.ptype"
          >{{ item.name }}</pre
        >
        <v-icon
          small
          class="reset-to-default"
          v-if="!isSetToDefault(item)"
          @click="resetToDefault(item)"
          title="Reset to default value"
          >mdi-backup-restore</v-icon
        >
      </div>
    </template>
    <template #[`item.value`]="{ item }">
      <ResourcePropertyEditor
        :value="item.value"
        :ptype="item.ptype"
        @input="updateResourceProperty(item.name, $event)"
      ></ResourcePropertyEditor>
    </template>
    <template #expanded-item="{ headers, item }">
      <td class="text-start" :colspan="headers.length - 2">Default value:</td>
      <td class="text-center">
        <ResourcePropertyEditor
          v-model="item.default_value"
          :ptype="item.ptype"
          readonly
          class="flex-grow-1"
        ></ResourcePropertyEditor>
      </td>
    </template>
  </v-data-table>
</template>

<style scoped>
#resource-properties {
  max-height: 100%;
  overflow: auto;
}

.reset-to-default {
  margin-left: 0.5rem;
}

.changed::after {
  content: "*";
  color: red;
  font-weight: bold;
}
</style>

<script>
import {
  get_resource_properties,
  update_resource_properties,
} from "~/modules/api";

export default {
  name: "ResourceProperties",
  data() {
    return {
      loading: false,
      headers: [
        {
          text: "Category",
          value: "group",
          align: "start",
          sortable: true,
          groupable: true,
        },
        {
          text: "Property",
          value: "name",
          align: "start",
          sortable: true,
          groupable: false,
        },
        {
          text: "Value",
          value: "value",
          align: "center",
          groupable: false,
        },
        {
          value: "data-table-expand",
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
  },
  methods: {
    queryResourceProperties(resourceId) {
      this.loading = true;

      get_resource_properties(resourceId).then((resp) => {
        this.properties = resp.properties;
        this.loading = false;
      });
    },
    updateResourceProperty(name, value) {
      const id = this.resourceDescription.id;
      const version = this.resourceDescription.version;
      //this.$emit("resource-change", resource);

      this.loading = true;

      update_resource_properties(id, version, [
        { name: name, value: value },
      ]).then((resp) => {
        this.properties.forEach(function (property, i, properties) {
          resp.updated_properties.forEach(function (updatedProperty) {
            if (property.name == updatedProperty.name) {
              properties[i].value = updatedProperty.value;
            }
          });
        });

        this.loading = false;
      });
    },
    isSetToDefault(item) {
      return JSON.stringify(item.value) == JSON.stringify(item.default_value);
    },
    resetToDefault(item) {
      item.value = JSON.parse(JSON.stringify(item.default_value));
      this.updateResourceProperty(item.name, item.value);
    },
  },
  mounted() {},
};
</script>
