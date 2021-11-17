<template>
  <v-form>
    <v-btn icon @click="undoTransaction()">
      <v-icon>mdi-undo</v-icon>
    </v-btn>
    <v-btn icon @click="redoTransaction()">
      <v-icon>mdi-redo</v-icon>
    </v-btn>
  </v-form>
</template>

<script>
import { undoTransaction, redoTransaction } from "~/modules/api";

export default {
  name: "EditToolBar",

  methods: {
    async undoTransaction() {
      await undoTransaction();
    },
    async redoTransaction() {
      await redoTransaction();
    },
  },
};
</script>
