<!-- Modified verion of https://icon-sets.iconify.design/ic/outline-insert-drive-file/ -->
<script lang="ts">
  export let text: string;

  export let title: string | null = null;

  let klass: string | null = null;

  export { klass as class };

  export let textClass: string | undefined = undefined;

  $: truncatedText = text.length > 4 ? text.slice(0, 4) + "..." : text;
</script>

<svg
  class={klass}
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
  role="img"
  preserveAspectRatio="xMidYMid meet"
  viewBox="0 0 512 512"
>
  <g
    transform="translate(0,512) scale(0.1,-0.1)"
    fill="currentColor"
    stroke="none"
  >
    <title>{title}</title>
    <path
      d="M1154 4671 c-140 -48 -237 -147 -280 -290 -10 -33 -14 -105 -14 -257
0 -116 -3 -468 -7 -782 l-6 -572 -189 0 c-161 0 -195 -3 -238 -19 -78 -30
-134 -81 -172 -159 l-33 -67 0 -500 c0 -493 0 -501 22 -547 30 -66 104 -141
166 -169 48 -22 68 -24 248 -27 l197 -3 4 -252 c5 -221 8 -259 26 -308 49
-135 153 -235 284 -274 48 -14 201 -15 1419 -13 l1364 3 65 31 c135 65 229
190 250 334 6 38 10 602 10 1335 l0 1270 -643 643 -642 642 -890 -1 c-817 0
-894 -2 -941 -18z m1616 -936 l0 -535 535 0 535 0 0 -1175 0 -1175 -1280 0
-1280 0 0 215 0 215 1093 2 1092 3 52 24 c66 29 143 111 169 176 18 47 19 80
19 545 l0 495 -33 67 c-38 78 -94 129 -172 159 -48 18 -89 19 -1135 19 l-1085
0 0 750 0 750 745 0 745 0 0 -535z"
    />
  </g>
  <g>
    <title>{text}</title>
    <text
      class={textClass ? `${textClass} text` : "text"}
      x="45"
      y="345"
      font-size="110"
      font-weight="bold"
      width="50"
      fill="currentColor"
    >
      {truncatedText}
    </text>
  </g>
</svg>

<style lang="postcss">
  .text {
    @apply uppercase;
  }
</style>
