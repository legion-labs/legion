// We load only the languages we need, here typecsript
// TODO: Replace by the chosen scripting language

import "monaco-editor/esm/vs/language/typescript/monaco.contribution";
import "monaco-editor/esm/vs/basic-languages/monaco.contribution";

import { Environment } from "monaco-editor/esm/vs/editor/editor.api";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

// Take from the `monaco.d.ts` file provided with the `monaco-editor` npm module.
// The one provided doesn't compile so we just copy paste the type here
declare global {
  interface Window {
    MonacoEnvironment?: Environment | undefined;
  }
}

// From https://github.com/microsoft/monaco-editor/blob/main/docs/integrate-esm.md#using-vite
// In dev mode, this configuration works only in browsers that support Workers modules
// (https://developer.mozilla.org/en-US/docs/Web/API/Worker/Worker)
window.MonacoEnvironment = {
  getWorker(_, label) {
    if (label === "json") {
      return new jsonWorker();
    }

    if (label === "css" || label === "scss" || label === "less") {
      return new cssWorker();
    }

    if (label === "html" || label === "handlebars" || label === "razor") {
      return new htmlWorker();
    }

    if (label === "typescript" || label === "javascript") {
      return new tsWorker();
    }

    return new editorWorker();
  },
};
