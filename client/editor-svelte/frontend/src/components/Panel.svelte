<div class="flex flex-col w-full h-full">
  <div class="flex flex-row h-7">
    <div class="flex bg-black flex-shrink-0 rounded-tl-lg">
      <div
        class="
          flex flex-row
          items-center
          bg-gray-700
          px-2
          rounded-t-lg
          text-sm
        "
      >
        <slot name="header" />
      </div>
    </div>
    <div class="flex bg-gray-700 w-full rounded-tr-lg">
      <div class="bg-black rounded-tr-lg rounded-bl-lg w-full" />
    </div>
  </div>
  <div class="bg-gray-700 h-full w-full rounded-b-lg">
    <slot name="content" />
  </div>
</div>
