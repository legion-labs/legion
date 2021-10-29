<template>
  <div id="app">
    <img alt="logo" src="../assets/logo.png">
    <ProcessList/>
  </div>
</template>

<script>
// @ is an alias to /src
import ProcessList from '@/components/process-list.vue'

export default {
  name: 'Home',
  components: {
    ProcessList
  }
}
</script>
