<script>
import { getResourceProperties, updateResourceProperties } from "~/modules/api";

export default {
  name: "ResourceProperties",
  // eslint-disable-next-line vue/prop-name-casing, vue/require-prop-types
  props: ["resource-description"],
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
  watch: {
    resourceDescription: {
      immediate: true,
      handler(val) {
        this.queryResourceProperties(val.id);
      },
    },
  },
  mounted() {},
  methods: {
    queryResourceProperties(resourceId) {
      this.loading = true;

      getResourceProperties(resourceId).then((resp) => {
        this.properties = resp.properties;
        this.loading = false;
      });
    },
    updateResourceProperty(name, value) {
      const id = this.resourceDescription.id;
      const version = this.resourceDescription.version;
      // this.$emit("resource-change", resource);

      this.loading = true;

      updateResourceProperties(id, version, [{ name, value }]).then((resp) => {
        this.properties.forEach(function (property, i, properties) {
          resp.updated_properties.forEach(function (updatedProperty) {
            if (property.name === updatedProperty.name) {
              properties[i].value = updatedProperty.value;
            }
          });
        });

        this.loading = false;
      });
    },
    isSetToDefault(item) {
      return JSON.stringify(item.value) === JSON.stringify(item.default_value);
    },
    resetToDefault(item) {
      item.value = JSON.parse(JSON.stringify(item.default_value));
      this.updateResourceProperty(item.name, item.value);
    },
  },
};
</script>

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
          v-if="!isSetToDefault(item)"
          small
          class="reset-to-default"
          title="Reset to default value"
          @click="resetToDefault(item)"
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
    <template #expanded-item="{ headers: dataTableHeaders, item }">
      <td class="text-start" :colspan="dataTableHeaders.length - 2">
        Default value:
      </td>
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
