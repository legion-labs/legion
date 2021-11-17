<template>
  <json-viewer v-if="readonly" :data="localValue"></json-viewer>
  <v-edit-dialog v-else large persistent :return-value.sync="localValue">
    <json-viewer :data="localValue"></json-viewer>
    <template #input>
      <json-editor
        :data-input="localValue"
        @data-output="(data) => (localValue = data)"
      ></json-editor>
    </template>
  </v-edit-dialog>
</template>

<script>
export default {
  name: "JSONWidget",
  props: {
    // eslint-disable-next-line vue/require-default-prop, vue/require-prop-types
    value: {},
    readonly: {
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
