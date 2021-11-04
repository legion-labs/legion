<template>
  <div>
    <div>process_id {{ process_id }}</div>
    <template v-for="process in process_list">
      <div :key="process.getProcessId()">exe {{ process.getExe() }}</div>
    </template>
    <template v-for="stream in stream_list">
      <div :key="stream.getStreamId()">Stream {{ stream.getStreamId() }}</div>
    </template>
    <template v-for="block in block_list">
      <div :key="block.getBlockId()">Block {{ block.getBlockId() }}</div>
    </template>
  </div>
</template>

<script>
import { BlockSpansRequest, ListStreamBlocksRequest, ListProcessStreamsRequest, FindProcessRequest, PerformanceAnalyticsClient } from '../proto/analytics_grpc_web_pb'

function fetchBlockSpans (block) {
  const streamId = block.getStreamId()
  const stream = this.stream_list.find(stream => stream.getStreamId() === streamId)
  const process = this.process_list.find(process => process.getProcessId() === stream.getProcessId())
  const request = new BlockSpansRequest()
  request.setProcess(process)
  request.setStream(stream)
  request.setBlockId(block.getBlockId())
  this.client.block_spans(request, null, (err, response) => {
    if (err) {
      console.error('error in block_call_tree', err)
    } else {
      console.log(response.toObject())
    }
  })
  console.log(request)
}

function fetchBlocks (streamId) {
  try {
    var request = new ListStreamBlocksRequest()
    request.setStreamId(streamId)
    this.client.list_stream_blocks(request, null, (err, response) => {
      if (err) {
        console.error('error in list_stream_blocks', err)
      } else {
        const newBlocks = response.getBlocksList()
        this.block_list = this.block_list.concat(newBlocks)
        newBlocks.forEach(block => this.fetchBlockSpans(block))
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
        this.process_list.push(response.getProcess())
        this.fetchStreams()
      }
    })
  } catch (err) {
    console.error(err.message)
    throw err
  }
}

function onTimelineCreated () {
  this.client = new PerformanceAnalyticsClient('http://' + location.hostname + ':9090', null, null)
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
      process_list: [],
      stream_list: []
    }
  },
  methods: {
    fetchBlocks: fetchBlocks,
    fetchBlockSpans: fetchBlockSpans,
    fetchStreams: fetchStreams,
    fetchProcessInfo: fetchProcessInfo
  }
}
</script>
