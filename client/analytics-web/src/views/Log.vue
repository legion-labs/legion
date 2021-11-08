<template>
  <div>
    <div>process_id {{ process_id }}</div>
  </div>
</template>

<script>
export default {
  name: 'Log',
  props: {
    process_id: {
      type: String,
      default: 'no'
    }
  }
}
</script>
