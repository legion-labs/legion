<template>
  <div>
    <div>process_id {{ process_id }}</div>
    <template v-for="(entry, index) in log_entries_list">
      <div class="logentry" :key="index">
        <span class="logentrytime">{{ formatTime(entry.getTimeMs()) }}</span>
        <span>{{ entry.getMsg() }}</span>
      </div>
    </template>
  </div>
</template>

<script>
import { ProcessLogRequest, FindProcessRequest, PerformanceAnalyticsClient } from '../proto/analytics_grpc_web_pb'

function formatTime (ms) {
  const seconds = ms / 1000
  const secondsWhole = Math.floor(seconds)
  const secondsStr = String(secondsWhole % 60).padStart(2, '0')
  const secondsFraction = String(Math.round(ms % 1000)).padStart(3, '0')
  const minutes = secondsWhole / 60
  const minutesWhole = Math.floor(minutes)
  const minutesStr = String(minutesWhole).padStart(2, '0')
  const hours = minutesWhole / 60
  const hoursWhole = Math.floor(hours)
  const hoursStr = String(hoursWhole).padStart(2, '0')
  return hoursStr + ':' + minutesStr + ':' + secondsStr + '.' + secondsFraction
}

function onCreated () {
  this.client = new PerformanceAnalyticsClient('http://' + location.hostname + ':9090', null, null)
  this.fetchProcessInfo()
}

function fetchProcessInfo () {
  try {
    var request = new FindProcessRequest()
    request.setProcessId(this.process_id)
    this.client.find_process(request, null, (err, response) => {
      if (err) {
        console.error('error in find_process', err)
      } else {
        const process = response.getProcess()
        this.process_list.push(process)
        this.fetchLog(process)
      }
    })
  } catch (err) {
    console.error(err.message)
    throw err
  }
}

function fetchLog (process) {
  var request = new ProcessLogRequest()
  request.setProcess(process)
  this.client.list_process_log_entries(request, null, (err, response) => {
    if (err) {
      console.error('error in list_process_log_entries', err)
    } else {
      const newLogEntries = response.getEntriesList()
      this.log_entries_list = this.log_entries_list.concat(newLogEntries)
    }
  })
}

export default {
  name: 'Log',
  props: {
    process_id: {
      type: String,
      default: 'no'
    }
  },
  created: onCreated,
  data: function () {
    return {
      process_list: [],
      log_entries_list: []
    }
  },
  methods: {
    fetchProcessInfo: fetchProcessInfo,
    fetchLog: fetchLog,
    formatTime: formatTime
  }
}
</script>

<style scoped>
.logentry {
  text-align: left;
  background-color: #F0F0F0;
}

.logentrytime {
  font-weight: bold;
  padding-right: 20px;
}
</style>
