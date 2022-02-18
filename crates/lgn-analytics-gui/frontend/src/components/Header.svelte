<script lang="ts">
  import User from "./User.svelte";
  import { authClient } from "@lgn/web-client/src/lib/auth";
  import { onMount } from "svelte";
  import { link } from "svelte-navigator";
  import iconPath from "../../icons/128x128.png";

  let user: string | undefined;

  onMount(async () => {
    user = (await authClient.userInfo()).name;
  });
</script>

<div class="w-full flex justify-between pl-6 pt-4 pr-4">
  <div class="flex items-center gap-3">
    <img src={iconPath} alt="logo" style="height:24px" class="inline" />
    <span class="font-bold text-xl">
      <a href="/" use:link>Legion Performance Analytics</a>
    </span>
  </div>
  {#if user}
    <div class="flex justify-between items-center">
      <User {user} />
    </div>
  {/if}
</div>
