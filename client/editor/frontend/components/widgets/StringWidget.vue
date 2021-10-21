<template>
  <pre v-if="readonly" :title="localValue">{{ localValue }}</pre>
  <v-text-field
    v-else-if="direct"
    single-line
    dense
    filled
    outlined
    hide-details
    type="string"
    v-model="localValue"
  ></v-text-field>
  <v-edit-dialog
    large
    persistent
    dark
    save-text="Apply"
    :return-value.sync="localValue"
    v-else
  >
    <pre :title="localValue">{{ localValue }}</pre>
    <template #input>
      <v-text-field
        type="string"
        v-model="localValue"
        single-line
        dense
        filled
        outlined
        hide-details
      ></v-text-field>
    </template>
  </v-edit-dialog>
</template>

<style scoped>
pre {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 4rem;
}
</style>

<script>
export default {
  name: "StringWidget",
  props: {
    value: String,
    readonly: {
      type: Boolean,
      default: false,
    },
    direct: {
      type: Boolean,
      default: false,
    },
  },
  computed: {
    localValue: {
      get() {
        return this.value;
      },
      set(val) {
        this.$emit("input", val);
      },
    },
  },
};
</script>