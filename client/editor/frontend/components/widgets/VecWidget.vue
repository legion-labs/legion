<template>
  <VecViewer v-if="readonly" v-model="localValue"></VecViewer>
  <VecEditor v-else-if="direct" v-model="localValue"></VecEditor>
  <v-edit-dialog
    large
    persistent
    dark
    save-text="Apply"
    :return-value.sync="localValue"
    v-else
  >
    <VecViewer v-model="localValue"></VecViewer>
    <template #input>
      <VecEditor v-model="localValue"></VecEditor>
    </template>
  </v-edit-dialog>
</template>

<script>
export default {
  name: "VecWidget",
  props: {
    value: Array,
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