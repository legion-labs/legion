import "monaco-editor/esm/vs/basic-languages/monaco.contribution";

import type { Environment } from "monaco-editor/esm/vs/editor/editor.api";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";

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
  getWorker() {
    return new editorWorker();
  },
};
