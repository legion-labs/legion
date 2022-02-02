<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from "svelte";
  import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
  import { debounce } from "../lib/promises";

  const dispatch = createEventDispatcher<{
    change: string;
  }>();

  const debounceTime = 500;

  export let theme: monaco.editor.BuiltinTheme | undefined = undefined;

  export let value: string;

  let editorContainer: HTMLDivElement | undefined;

  let editor: monaco.editor.IStandaloneCodeEditor | undefined;

  onMount(() => {
    if (!editorContainer) {
      return;
    }

    editor = monaco.editor.create(editorContainer, {
      value,
      language: "rust",
      automaticLayout: true,
      theme,
    });

    editor.onDidChangeModelContent(
      debounce(() => {
        dispatch("change", getValue());
      }, debounceTime)
    );
  });

  onDestroy(() => {
    editor?.dispose();
  });

  /** Returns the current value of the editor (as a string)
   * `undefined` is returned if the editor doesn't exist.
   */
  export function getValue() {
    return editor?.getValue() ?? "";
  }
</script>

<div class="root" bind:this={editorContainer} />

<style lang="postcss">
  .root {
    @apply h-full w-full;
  }
</style>
