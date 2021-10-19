<template>
  <ColorViewer v-model="localValue" v-if="readonly"></ColorViewer>
  <v-edit-dialog
    large
    persistent
    dark
    save-text="Apply"
    :return-value.sync="localValue"
    v-else
  >
    <ColorViewer v-model="localValue"></ColorViewer>
    <template #input>
      <ColorEditor v-model="localValue"></ColorEditor>
    </template>
  </v-edit-dialog>
</template>

<script>
export default {
  name: "ColorWidget",
  props: {
    value: String,
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