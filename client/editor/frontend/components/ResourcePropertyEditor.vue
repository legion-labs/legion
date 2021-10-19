<template>
  <v-edit-dialog :return-value.sync="value" v-if="ptype == 'color'">
    <v-chip>{{ value }}</v-chip>
    <template #input>
      <v-text-field
        v-model="localValue"
        label="Edit"
        single-line
      ></v-text-field>
    </template>
  </v-edit-dialog>
  <v-edit-dialog :return-value.sync="value" v-else>
    <vue-json-pretty :data="value"></vue-json-pretty>
  </v-edit-dialog>
</template>

<script>
export default {
  name: "ResourcePropertyEditor",
  props: ["value", "ptype"],
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
