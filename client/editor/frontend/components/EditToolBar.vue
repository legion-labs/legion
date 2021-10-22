<template>
  <v-form>
    <v-btn icon @click="undo_transaction()">
      <v-icon>mdi-undo</v-icon>
    </v-btn>
    <v-btn icon @click="redo_transaction()">
      <v-icon>mdi-redo</v-icon>
    </v-btn>
  </v-form>
</template>

<script>
import { undo_transaction, redo_transaction } from "~/modules/api";

export default {
  name: "EditToolBar",

  methods: {
    async undo_transaction() {
      await undo_transaction();
    },
    async redo_transaction() {
      await redo_transaction();
    },
  },
};
</script>