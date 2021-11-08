<template>
<div class="ProcessList">
  <h1>Legion Performance Analytics</h1>
  <h2>Process List</h2>
  <center>
    <table>
      <thead>
        <th>Start Time</th>
        <th>id</th>
        <th>exe</th>
        <th>parent id</th>
        <th>timeline</th>
      </thead>
      <tbody>
        <template v-for="process in process_list">
          <TR :key="process.getProcessInfo().getProcessId()">
            <td>{{ process.getProcessInfo().getStartTime() }}</td>
            <td>{{ process.getProcessInfo().getProcessId() }}</td>
            <td>{{ process.getProcessInfo().getExe() }}</td>
            <td>{{ process.getProcessInfo().getParentProcessId() }}</td>
            <td>
              <div v-if="process.getNbCpuBlocks() > 0">
                <router-link v-bind:to="{ name: 'Timeline', params: {process_id: process.getProcessInfo().getProcessId() } }">timeline</router-link>
              </div>
              <div v-if="process.getNbLogBlocks() > 0">
                <router-link v-bind:to="{ name: 'Log', params: {process_id: process.getProcessInfo().getProcessId() } }">log</router-link>
              </div>
            </td>
          </TR>
        </template>
      </tbody>
    </table>
  </center>
</div>
</template>

<script>
import { RecentProcessesRequest, PerformanceAnalyticsClient } from '../proto/analytics_grpc_web_pb'

export default {
  name: 'ProcessList',
  created: function () {
    this.client = new PerformanceAnalyticsClient('http://' + location.hostname + ':9090', null, null)
    try {
      var request = new RecentProcessesRequest()
      this.client.list_recent_processes(request, null, (err, response) => {
        if (err) {
          console.error('error in list_recent_processes', err)
        } else {
          this.process_list = response.getProcessesList()
        }
      })
    } catch (err) {
      console.error(err.message)
      throw err
    }
  },
  data: function () {
    return {
      process_list: []
    }
  }
}
</script>

<style scoped>
table {
    border-collapse: collapse;
}
table tbody {
    overflow: auto;
}
table thead
{
   background: rgb(230, 230, 230);
}
table th
{
   padding:     0 5px;
   text-align:  center;
   border:      1px solid rgb(153, 153, 153);
}
table td
{
   padding: 5px;
   text-align:  left;
   font-family: monospace;
   border:      1px solid rgb(153, 153, 153);
}
table td div{
   padding: 5px;
}
a {
    color: #42b983;
}
</style>
