<script lang="ts">
  async function fetchRestaurant() {
    const response = await fetch(
      "https://random-data-api.com/api/restaurant/random_restaurant"
    );
    return response.json();
  }

  let restaurant = fetchRestaurant();

  const findAnotherRestaurant = () => {
    restaurant = fetchRestaurant();
  };
</script>

<div class="root">
  <button class="button" on:click={findAnotherRestaurant}
    >Find another restaurant</button
  >
  {#await restaurant}
    <div>Loading...</div>
  {:then restaurant}
    <div>
      <div>Name: {restaurant.name}</div>
      <div>Type: {restaurant.type}</div>
      <div>Description: {restaurant.description}</div>
    </div>
  {/await}
</div>

<style lang="postcss">
  .root {
    @apply h-screen w-full text-gray-800 p-4;
  }

  .button {
    @apply px-2 py-1 rounded bg-gray-200 border border-gray-300 hover:bg-gray-300 transition;
  }
</style>
