<template>
  <div>
    <template v-for="process in process_list">
      <div :key="process.getProcessId()">{{ process.getExe() }} {{ process.getProcessId() }}
        <div v-if="process.getParentProcessId() != ''">
          <router-link v-bind:to="{ name: 'Timeline', params: {process_id: process.getParentProcessId() } }">Parent timeline</router-link>
        </div>
      </div>
    </template>
    <template v-for="stream in stream_list">
      <div :key="stream.getStreamId()">Stream {{ stream.getStreamId() }}</div>
    </template>
    <template v-for="block in block_list">
      <div :key="block.getBlockId()">Block {{ block.getBlockId() }}</div>
    </template>
    <canvas id="canvas_timeline" width="1024px" height="640px"></canvas>
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
      console.error('error in block_spans', err)
    } else {
      response.getScopesList().forEach(scopeDesc => {
        this.scopes[scopeDesc.getHash()] = scopeDesc
      })
      this.span_block_list.push(response)
    }
  })
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
        console.error('error in find_process', err)
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
  this.$watch(
    () => this.$route.params,
    (toParams, previousParams) => {
      this.process_id = toParams.process_id
      this.block_list = []
      this.process_list = []
      this.span_block_list = []
      this.scopes = {}
      this.stream_list = []
      this.fetchProcessInfo()
    }
  )
  this.client = new PerformanceAnalyticsClient('http://' + location.hostname + ':9090', null, null)
  this.fetchProcessInfo()
}

function onMounted () {
  const canvas = document.getElementById('canvas_timeline')
  this.renderingContext = canvas.getContext('2d')
}

function drawCanvas () {
  const begin = Math.min(...this.span_block_list.map(block => block.getBeginMs()))
  const end = Math.max(...this.span_block_list.map(block => block.getEndMs()))
  const invTimeSpan = 1.0 / (end - begin)
  const canvas = document.getElementById('canvas_timeline')
  const canvasWidth = canvas.clientWidth
  const msToPixelsFactor = invTimeSpan * canvasWidth
  this.renderingContext.font = '15px arial'

  const testString = '<>_w'
  const testTextMetrics = this.renderingContext.measureText(testString)
  const characterWidth = testTextMetrics.width / testString.length
  const characterHeight = testTextMetrics.actualBoundingBoxAscent
  this.span_block_list.forEach(blockSpans => {
    blockSpans.getSpansList().forEach(span => {
      const beginPixels = (span.getBeginMs() - begin) * msToPixelsFactor
      const endPixels = (span.getEndMs() - begin) * msToPixelsFactor
      const callWidth = endPixels - beginPixels
      const depth = span.getDepth()
      const offsetY = depth * 20
      if (depth % 2 === 0) {
        this.renderingContext.fillStyle = '#7DF9FF'
      } else {
        this.renderingContext.fillStyle = '#A0A0CC'
      }
      this.renderingContext.fillRect(beginPixels, offsetY, callWidth, 20)
      const scope = this.scopes[span.getScopeHash()]
      const name = scope.getName()
      if (callWidth > (characterWidth * 5)) {
        const nbChars = Math.floor(callWidth / characterWidth)
        this.renderingContext.fillStyle = '#000000'
        const extraHeight = 0.5 * (20 - characterHeight)
        this.renderingContext.fillText(name.slice(0, nbChars), beginPixels + 5, offsetY + characterHeight + extraHeight, callWidth)
      }
    })
  })
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
  mounted: onMounted,
  data: function () {
    return {
      block_list: [],
      process_list: [],
      span_block_list: [],
      scopes: {},
      stream_list: []
    }
  },
  watch: {
    span_block_list: drawCanvas
  },
  methods: {
    fetchBlocks: fetchBlocks,
    fetchBlockSpans: fetchBlockSpans,
    fetchStreams: fetchStreams,
    fetchProcessInfo: fetchProcessInfo,
    drawCanvas: drawCanvas
  }
}
</script>
