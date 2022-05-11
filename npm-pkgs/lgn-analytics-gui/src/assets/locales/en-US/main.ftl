# Global
global-pagination-first = First
global-pagination-previous = Previous
global-pagination-last = Last
global-pagination-next = Next
global-platform =
  {$platform ->
    [linux] Linux
    [windows] Windows
    *[unknown] Unknown
  }
global-link-copy = Copy Link
global-link-share = Share Link
global-cumulative-call-graph = Cumulative Call Graph
global-log = Log
global-timeline = Timeline
global-thread = thread

# Process list
process-list-user = User
process-list-process = Process
process-list-computer = Computer
process-list-platform = Plateform
process-list-start-time = Start Time
process-list-statistics = Statistics
process-list-search = Search Process...

# Log
log-process-id = Process Id:
log-executable = Executable:
log-parent-link = Parent Process Log

# Timeline
timeline-open-cumulative-call-graph = Open { global-cumulative-call-graph }
timeline-search = Search...
timeline-table-function = Function
timeline-table-count = Count
timeline-table-average = Avg
timeline-table-minimum = Min
timeline-table-maximum = Max
timeline-table-standard-deviation = Sd
timeline-table-sum = Sum
timeline-main-collapsed-extra =
  {$validThreadCount ->
    [0] (No thread data)
    *[other] ({$validThreadCount} {$validThreadCount ->
      [one] thread
      *[other] threads
    } with data)
  }
timeline-main-thread-description-title =
  {$threadName}
  {$threadLength}
  {$threadBlocks} {$threadBlocks ->
    [one] block
    *[other] blocks
  }
timeline-main-thread-description =
  {$threadLength} ({$threadBlocks} {$threadBlocks ->
    [one] block
    *[other] blocks
  })
timeline-main-collapse = Collapse
timeline-main-expand = Expand
timeline-debug-tooltip =
  Pixel size: { $pixelSize }
  Lod: { $lod }
  Threshold: { $threshold }
  Events: { $events }
timeline-link-copy-notification-title = Copy Succeeded
timeline-link-copy-notification-message = The link has been copied to your clipboard successfully
