<template>
  <json-viewer :data="localValue" v-if="readonly"></json-viewer>
  <v-edit-dialog large persistent :return-value.sync="localValue" v-else>
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