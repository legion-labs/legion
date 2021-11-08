<template>
  <div>
    <div>process_id {{ process_id }}</div>
  </div>
</template>

<script>
import { ProcessLogRequest, FindProcessRequest, PerformanceAnalyticsClient } from '../proto/analytics_grpc_web_pb'

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
      console.log(response.getEntriesList())
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
      process_list: []
    }
  },
  methods: {
    fetchProcessInfo: fetchProcessInfo,
    fetchLog: fetchLog
  }
}
</script>
