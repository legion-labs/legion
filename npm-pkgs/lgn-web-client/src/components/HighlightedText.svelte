<script lang="ts">
  export let text: string;

  export let pattern: RegExp | string | (RegExp | string)[];

  function highlightText(text: string, pattern: RegExp | string) {
    return text.replaceAll(
      pattern,
      (match) =>
        `<mark class="bg-orange-700 text-black font-bold">${match}</mark>`
    );
  }

  $: highlightedText = Array.isArray(pattern)
    ? pattern.reduce(highlightText, text)
    : highlightText(text, pattern);
</script>

{@html highlightedText}
