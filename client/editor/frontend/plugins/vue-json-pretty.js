import Vue from "vue";
import VueJsonPretty from "vue-json-pretty";
import JsonEditor from "@kassaila/vue-json-editor";

Vue.component("JsonViewer", VueJsonPretty);
Vue.component("JsonEditor", JsonEditor);
