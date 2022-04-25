<script lang="ts">
  import User from "../List/User.svelte";
  import { authClient } from "@lgn/web-client/src/lib/auth";
  import { onMount } from "svelte";
  import { link } from "svelte-navigator";
  import iconPath from "../../../icons/128x128.png";

  let user: string | undefined;
  let dark = true;

  $: document.body.classList.toggle("dark", dark);

  onMount(async () => {
    user = (await authClient.userInfo()).name;
  });
</script>

<div class="w-full flex justify-between pl-6 pt-4 pr-4">
  <div class="flex items-center gap-3">
    <a href="/" use:link>
      <img
        src={iconPath}
        alt="logo"
        style="height:24px"
        class="inline"
        on:click={(e) => {
          if (e.shiftKey && e.ctrlKey) {
            e.preventDefault();
            dark = !dark;
          }
        }}
      />
      <span class="header-logo">
        <a href="/" use:link>Legion Performance Analytics</a>
      </span>
    </a>
  </div>
  {#if user}
    <div class="flex justify-between items-center">
      <User {user} />
    </div>
  {/if}
</div>

<style lang="postcss">
  .pr-4 {
    background-color: #f6f6f6;
    padding: 0.75rem 0.75rem;
    box-shadow: 0 1px 1px 0 rgb(255, 255, 255), 0 2px 1px 0 rgb(230, 234, 238);
  }

  a {
    color: #000000;
  }

  .header-logo {
  @apply font-default text-base;
  }

</style>
