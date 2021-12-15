<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import * as monaco from "monaco-editor";

  export let theme: monaco.editor.BuiltinTheme | undefined = undefined;

  let editorContainer: HTMLDivElement | undefined;

  let editor: monaco.editor.IStandaloneCodeEditor | undefined;

  onMount(() => {
    editor = monaco.editor.create(editorContainer!, {
      value: 'function hello(): void {\n\talert("Hello Legion");\n}\n',
      language: "typescript",
      automaticLayout: true,
      theme,
    });
  });

  onDestroy(() => {
    editor?.dispose();
  });

  /** Returns the current value of the editor (as a string)
   * `undefined` is returned if the editor doesn't exist.
   */
  export function getValue() {
    return editor?.getValue();
  }
</script>

<div class="root" bind:this={editorContainer} />

<style lang="postcss">
  .root {
    @apply h-full w-full;
  }
</style>
