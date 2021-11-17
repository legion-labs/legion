<template>
  <VecViewer v-if="readonly" v-model="localValue"></VecViewer>
  <VecEditor v-else-if="direct" v-model="localValue"></VecEditor>
  <v-edit-dialog
    v-else
    large
    persistent
    dark
    save-text="Apply"
    :return-value.sync="localValue"
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
    // eslint-disable-next-line vue/require-default-prop
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
      set(valarray) {
        const valNums = [];
        for (const val of valarray) {
          const valNum = parseFloat(val);
          if (isNaN(valNum)) {
            return;
          }
          valNums.push(valNum);
        }
        this.$emit("input", valNums);
      },
    },
  },
};
</script>
