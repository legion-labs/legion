<template>
  <div>
    <template v-for="process in process_list">
      <div :key="process.getProcessId()">{{ process.getExe() }} {{ process.getProcessId() }}
  <div v-if="process.getParentProcessId() != ''">
          <router-link v-bind:to="{ name: 'Timeline', params: {process_id: process.getParentProcessId() } }">Parent timeline</router-link>
        </div>
      </div>
    </template>
    <canvas id="canvas_timeline"
            width="1024px"
            height="640px"
            v-on:wheel.prevent="onZoom"
            v-on:mousemove="onPan"
            />
  </div>
</template>

<script>
import { BlockSpansRequest, ListStreamBlocksRequest, ListProcessStreamsRequest, FindProcessRequest, PerformanceAnalyticsClient } from '../proto/analytics_grpc_web_pb'

function fetchBlockSpans (block) {
  const streamId = block.getStreamId()
  const stream = this.threads[streamId].streamInfo
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
      this.min_ms = Math.min(this.min_ms, response.getBeginMs())
      this.max_ms = Math.max(this.max_ms, response.getEndMs())
      this.threads[streamId].spanBlocks.push(response)
      this.drawCanvas()
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
        response.getStreamsList().forEach(stream => {
          if (stream.getTagsList().includes('cpu')) {
            this.threads[stream.getStreamId()] = {
              streamInfo: stream,
              spanBlocks: []
            }
            this.fetchBlocks(stream.getStreamId())
          }
        })
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
      this.reset(toParams.process_id)
    }
  )
  this.client = new PerformanceAnalyticsClient('http://' + location.hostname + ':9090', null, null)
  this.fetchProcessInfo()
}

function onMounted () {
  const canvas = document.getElementById('canvas_timeline')
  this.renderingContext = canvas.getContext('2d')
}

function formatExecutionTime (time) {
  let unit = 'ms'
  if (time < 1) {
    unit = 'us'
    time = time * 1000
    return time.toFixed(3) + ' ' + unit
  }
  if (time > 1000) {
    unit = 'seconds'
    time = time / 1000
  }
  return time.toFixed(3) + ' ' + unit
}

function drawThread (thread, threadVerticalOffset) {
  const viewRange = this.getViewRange()
  const begin = viewRange[0]
  const end = viewRange[1]
  const invTimeSpan = 1.0 / (end - begin)
  const canvas = document.getElementById('canvas_timeline')
  const canvasWidth = canvas.clientWidth
  const msToPixelsFactor = invTimeSpan * canvasWidth
  this.renderingContext.font = '15px arial'
  const testString = '<>_w'
  const testTextMetrics = this.renderingContext.measureText(testString)
  const characterWidth = testTextMetrics.width / testString.length
  const characterHeight = testTextMetrics.actualBoundingBoxAscent
  thread.spanBlocks.forEach(blockSpans => {
    blockSpans.getSpansList().forEach(span => {
      const beginPixels = (span.getBeginMs() - begin) * msToPixelsFactor
      const endPixels = (span.getEndMs() - begin) * msToPixelsFactor
      const callWidth = endPixels - beginPixels
      const depth = span.getDepth()
      const offsetY = threadVerticalOffset + (depth * 20)
      if (depth % 2 === 0) {
        this.renderingContext.fillStyle = '#7DF9FF'
      } else {
        this.renderingContext.fillStyle = '#A0A0CC'
      }
      this.renderingContext.fillRect(beginPixels, offsetY, callWidth, 20)
      const scope = this.scopes[span.getScopeHash()]
      if (callWidth > (characterWidth * 5)) {
        const nbChars = Math.floor(callWidth / characterWidth)
        this.renderingContext.fillStyle = '#000000'
        const extraHeight = 0.5 * (20 - characterHeight)
        const name = scope.getName()
        const caption = name + ' ' + formatExecutionTime(span.getEndMs() - span.getBeginMs())
        this.renderingContext.fillText(caption.slice(0, nbChars), beginPixels + 5, offsetY + characterHeight + extraHeight, callWidth)
      }
    })
  })
}

function drawCanvas () {
  const canvas = document.getElementById('canvas_timeline')
  this.renderingContext.clearRect(0, 0, canvas.width, canvas.height)
  let threadVerticalOffset = 0
  for (const streamId in this.threads) {
    this.drawThread(this.threads[streamId], threadVerticalOffset)
    threadVerticalOffset += 110
  }
}

function onPan (evt) {
  if (evt.buttons !== 1) {
    this.begin_drag = undefined
    return
  }
  if (!this.begin_drag) {
    this.begin_drag = {
      beginMouseX: evt.offsetX,
      beginMouseY: evt.offsetY,
      viewRange: this.getViewRange()
    }
  }
  const canvas = document.getElementById('canvas_timeline')
  const factor = (this.begin_drag.viewRange[1] - this.begin_drag.viewRange[0]) / canvas.width
  const offsetMs = factor * (this.begin_drag.beginMouseX - evt.offsetX)
  this.view_range = [this.begin_drag.viewRange[0] + offsetMs, this.begin_drag.viewRange[1] + offsetMs]
  this.drawCanvas()
}

function onZoom (evt) {
  const speed = 1.25
  const factor = evt.wheelDeltaY > 0 ? (1.0 / speed) : speed
  const oldRange = this.getViewRange()
  const length = oldRange[1] - oldRange[0]
  const newLength = length * factor
  const canvas = document.getElementById('canvas_timeline')
  const pctCursor = evt.offsetX / canvas.width
  const pivot = oldRange[0] + (length * pctCursor)
  this.view_range = [pivot - (newLength * pctCursor), pivot + (newLength * (1 - pctCursor))]
  this.drawCanvas()
}

function getViewRange () {
  if (this.view_range) {
    return this.view_range
  }
  return [this.min_ms, this.max_ms]
}

function reset (processId) {
  this.process_id = processId
  this.block_list = []
  this.process_list = []
  this.scopes = {}
  this.threads = []
  this.min_ms = Infinity
  this.max_ms = -Infinity
  this.view_range = undefined
  this.begin_drag = undefined
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
  mounted: onMounted,
  data: function () {
    return {
      block_list: [],
      process_list: [],
      scopes: {},
      threads: {},
      min_ms: Infinity,
      max_ms: -Infinity,
      view_range: undefined,
      begin_drag: undefined
    }
  },
  methods: {
    drawCanvas: drawCanvas,
    drawThread: drawThread,
    fetchBlockSpans: fetchBlockSpans,
    fetchBlocks: fetchBlocks,
    fetchProcessInfo: fetchProcessInfo,
    fetchStreams: fetchStreams,
    getViewRange: getViewRange,
    onPan: onPan,
    onZoom: onZoom,
    reset: reset
  }
}
</script>
