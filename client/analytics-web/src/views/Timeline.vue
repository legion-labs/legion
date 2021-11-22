<template>
  <div>
    <template v-bind:process="current_process">
      <div v-if="current_process">
        <div>{{ current_process.getExe() }} {{ current_process.getProcessId() }}</div>
        <div v-if="current_process.getParentProcessId() != ''">
          <router-link
            v-bind:to="{ name: 'Timeline', params: {process_id: current_process.getParentProcessId() } }">
            Parent timeline
          </router-link>
        </div>
      </div>
    </template>
    <canvas id="canvas_timeline"
            width="1024px"
            height="768px"
            v-on:wheel.prevent="onZoom"
            v-on:mousemove="onMouseMove"
            />
  </div>
</template>

<script>
import { ListProcessChildrenRequest, BlockSpansRequest, ListStreamBlocksRequest, ListProcessStreamsRequest, FindProcessRequest, PerformanceAnalyticsClient } from '../proto/analytics_grpc_web_pb'

function findStreamProcess (streamId) {
  const stream = this.threads[streamId].streamInfo
  const process = this.process_list.find(process => process.getProcessId() === stream.getProcessId())
  return process
}

function fetchBlockSpans (block) {
  const streamId = block.getStreamId()
  const stream = this.threads[streamId].streamInfo
  const process = this.findStreamProcess(streamId)
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

function fetchStreams (process) {
  try {
    var request = new ListProcessStreamsRequest()
    request.setProcessId(process.getProcessId())
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

function fetchChildren () {
  var listChildrenRequest = new ListProcessChildrenRequest()
  listChildrenRequest.setProcessId(this.process_id)
  this.client.list_process_children(listChildrenRequest, null, (err, response) => {
    if (err) {
      console.error('error in list_process_children', err)
    } else {
      response.getProcessesList().forEach(process => {
        this.process_list.push(process)
        this.fetchStreams(process)
      })
    }
  })
}

function fetchProcessInfo () {
  try {
    var findProcessRequest = new FindProcessRequest()
    findProcessRequest.setProcessId(this.process_id)
    this.client.find_process(findProcessRequest, null, (err, response) => {
      if (err) {
        console.error('error in find_process', err)
      } else {
        const process = response.getProcess()
        this.process_list.push(process)
        this.fetchStreams(process)
        this.current_process = process
        this.fetchChildren()
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

function drawThread (thread, threadVerticalOffset, offsetMs) {
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
  let maxDepth = 0
  thread.spanBlocks.forEach(blockSpans => {
    maxDepth = Math.max(maxDepth, blockSpans.getMaxDepth())
    blockSpans.getSpansList().forEach(span => {
      const beginSpan = span.getBeginMs() + offsetMs
      const endSpan = span.getEndMs() + offsetMs
      const beginPixels = (beginSpan - begin) * msToPixelsFactor
      const endPixels = (endSpan - begin) * msToPixelsFactor
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
        const caption = name + ' ' + formatExecutionTime(endSpan - beginSpan)
        this.renderingContext.fillText(caption.slice(0, nbChars), beginPixels + 5, offsetY + characterHeight + extraHeight, callWidth)
      }
    })
  })
  return maxDepth
}

function drawSelectedRange () {
  if (this.selected_range === undefined) {
    return
  }
  const viewRange = this.getViewRange()
  const begin = viewRange[0]
  const end = viewRange[1]
  const invTimeSpan = 1.0 / (end - begin)
  const canvas = document.getElementById('canvas_timeline')
  const canvasWidth = canvas.clientWidth
  const canvasHeight = canvas.clientHeight
  const msToPixelsFactor = invTimeSpan * canvasWidth
  const beginSelection = this.selected_range[0]
  const endSelection = this.selected_range[1]
  const beginPixels = (beginSelection - begin) * msToPixelsFactor
  const endPixels = (endSelection - begin) * msToPixelsFactor
  this.renderingContext.fillStyle = 'rgba(64, 64, 200, 0.2)'
  this.renderingContext.fillRect(beginPixels, 0, endPixels - beginPixels, canvasHeight)
}

function drawCanvas () {
  const canvas = document.getElementById('canvas_timeline')
  canvas.height = window.innerHeight - canvas.getBoundingClientRect().top - 20
  this.renderingContext.clearRect(0, 0, canvas.width, canvas.height)
  let threadVerticalOffset = this.y_offset
  const parentStartTime = Date.parse(this.current_process.getStartTime())
  for (const streamId in this.threads) {
    const childProcess = this.findStreamProcess(streamId)
    const childStartTime = Date.parse(childProcess.getStartTime())
    const maxDepth = this.drawThread(this.threads[streamId], threadVerticalOffset, childStartTime - parentStartTime)
    threadVerticalOffset += (maxDepth + 2) * 20
  }
  this.drawSelectedRange()
}

function onPan (evt) {
  if (!this.begin_pan) {
    this.begin_pan = {
      beginMouseX: evt.offsetX,
      beginMouseY: evt.offsetY,
      viewRange: this.getViewRange(),
      beginYOffset: this.y_offset
    }
  }
  const canvas = document.getElementById('canvas_timeline')
  const factor = (this.begin_pan.viewRange[1] - this.begin_pan.viewRange[0]) / canvas.width
  const offsetMs = factor * (this.begin_pan.beginMouseX - evt.offsetX)
  this.view_range = [this.begin_pan.viewRange[0] + offsetMs, this.begin_pan.viewRange[1] + offsetMs]
  this.y_offset = this.begin_pan.beginYOffset + evt.offsetY - this.begin_pan.beginMouseY
  this.drawCanvas()
}

function onSelectRange (evt) {
  if (!this.begin_select) {
    this.begin_select = {
      beginMouseX: evt.offsetX
    }
  }
  const canvas = document.getElementById('canvas_timeline')
  const viewRange = this.getViewRange()
  const factor = (viewRange[1] - viewRange[0]) / canvas.width
  const beginTime = viewRange[0] + (factor * this.begin_select.beginMouseX)
  const endTime = viewRange[0] + (factor * evt.offsetX)
  this.selected_range = [beginTime, endTime]
  this.drawCanvas()
}

function onMouseMove (evt) {
  if (evt.buttons !== 1) {
    this.begin_pan = undefined
    this.begin_select = undefined
    return
  }
  if (evt.shiftKey) {
    this.onSelectRange(evt)
  } else {
    this.onPan(evt)
  }
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
  this.current_process = undefined
  this.scopes = {}
  this.threads = []
  this.min_ms = Infinity
  this.max_ms = -Infinity
  this.view_range = undefined
  this.begin_pan = undefined
  this.begin_select = undefined
  this.y_offset = 0
  this.selected_range = undefined
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
      current_process: undefined,
      scopes: {},
      threads: {},
      min_ms: Infinity,
      max_ms: -Infinity,
      view_range: undefined,
      begin_pan: undefined,
      begin_select: undefined,
      y_offset: 0,
      selected_range: undefined
    }
  },
  methods: {
    drawCanvas: drawCanvas,
    drawSelectedRange: drawSelectedRange,
    drawThread: drawThread,
    fetchBlockSpans: fetchBlockSpans,
    fetchBlocks: fetchBlocks,
    fetchChildren: fetchChildren,
    fetchProcessInfo: fetchProcessInfo,
    fetchStreams: fetchStreams,
    findStreamProcess: findStreamProcess,
    getViewRange: getViewRange,
    onMouseMove: onMouseMove,
    onPan: onPan,
    onSelectRange: onSelectRange,
    onZoom: onZoom,
    reset: reset
  }
}
</script>
