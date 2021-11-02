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
          <TR :key="process.getProcessId()">
            <td>{{ process.getStartTime() }}</td>
            <td>{{ process.getProcessId() }}</td>
            <td>{{ process.getExe() }}</td>
            <td>{{ process.getParentProcessId() }}</td>
            <td>
              <p><router-link to="/timeline">timeline</router-link></p>
              <router-link to="/log">log</router-link>
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
    this.client = new PerformanceAnalyticsClient('http://localhost:9090', null, null)
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
   padding:     0 5px;
   text-align:  left;
   font-family: monospace;
   white-space: nowrap;
   border:      1px solid rgb(153, 153, 153);
}
a {
    color: #42b983;
}
</style>
