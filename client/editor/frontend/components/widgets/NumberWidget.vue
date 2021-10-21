<template>
  <pre v-if="readonly">{{ localValue }}</pre>
  <v-text-field
    v-else-if="direct"
    type="number"
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
    <pre>{{ localValue }}</pre>
    <template #input>
      <v-text-field type="number" v-model="localValue"></v-text-field>
    </template>
  </v-edit-dialog>
</template>

<script>
export default {
  name: "NumberWidget",
  props: {
    value: Number,
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