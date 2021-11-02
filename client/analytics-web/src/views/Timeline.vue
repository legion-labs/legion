<template>
  <div>
    <div>process_id {{ process_id }}</div>
    <div>exe {{ process_info.getExe() }}</div>
    <template v-for="stream in stream_list">
      <div :key="stream.getStreamId()">Stream {{ stream.getStreamId() }}</div>
    </template>
    <template v-for="block in block_list">
      <div :key="block.getBlockId()">Block {{ block.getBlockId() }}</div>
    </template>
  </div>
</template>

<script>
import { ListStreamBlocksRequest, ListProcessStreamsRequest, FindProcessRequest, PerformanceAnalyticsClient } from '../proto/analytics_grpc_web_pb'

function fetchBlocks (streamId) {
  try {
    var request = new ListStreamBlocksRequest()
    request.setStreamId(streamId)
    this.client.list_stream_blocks(request, null, (err, response) => {
      if (err) {
        console.error('error in list_stream_blocks', err)
      } else {
        this.block_list = this.block_list.concat(response.getBlocksList())
      }
    })
  } catch (err) {
    console.error(err.message)
    throw err
  }
}

function fetchStreams () {
  try {
    var request = new ListProcessStreamsRequest()
    request.setProcessId(this.process_id)
    this.client.list_process_streams(request, null, (err, response) => {
      if (err) {
        console.error('error in list_process_streams', err)
      } else {
        const filteredStreams = []
        response.getStreamsList().forEach(stream => {
          if (stream.getTagsList().includes('cpu')) {
            this.fetchBlocks(stream.getStreamId())
            filteredStreams.push(stream)
          }
        })
        this.stream_list = filteredStreams
      }
    })
  } catch (err) {
    console.error(err.message)
    throw err
  }
}

function fetchProcessInfo () {
  try {
    var request = new FindProcessRequest()
    request.setProcessId(this.process_id)
    this.client.find_process(request, null, (err, response) => {
      if (err) {
        console.error('error in list_process_streams', err)
      } else {
        this.process_info = response.getProcess()
      }
    })
  } catch (err) {
    console.error(err.message)
    throw err
  }
}

function onTimelineCreated () {
  this.client = new PerformanceAnalyticsClient('http://' + location.hostname + ':9090', null, null)
  this.fetchStreams()
  this.fetchProcessInfo()
}

export default {
  name: 'Timeline',
  props: {
    process_id: {
      type: String,
      default: 'no'
    }
  },
  created: onTimelineCreated,
  data: function () {
    return {
      block_list: [],
      process_info: { getExe: function () { return '' } },
      stream_list: []
    }
  },
  methods: {
    fetchBlocks: fetchBlocks,
    fetchStreams: fetchStreams,
    fetchProcessInfo: fetchProcessInfo
  }
}
</script>
