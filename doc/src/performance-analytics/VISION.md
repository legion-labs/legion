---
marp: true
---

![bg](figures/health.jpg)
![](white)
# Legion Performance Analytics

Septembre 2021

---
# Legion Performance Analytics
- **Introduction**
- Requirements
- Architecture
- Roadmap

---

![bg opacity:.2](figures/moto.jpg)
# Introduction
## Definition
- `Performance`: quantified non-functional requirement
- **The test of the machine is the satisfaction it gives you.**― Robert M. Pirsig, _Zen and the Art of Motorcycle Maintenance: An Inquiry Into Values_
- Performance for software
  - Latency
    * ex.: frame time, reaction time, replication time, load time
  - Stability
    * ex.: MTBF, crashes, error logs, memory use
  - Satisfaction
    * ex.: retention, engagement, biometrics, surveys

---

# Introduction
## Stages of data storage

![bg right](figures/disk.jpg)
- **in-app**: buffered streams of structured events
- **Data Lake**
  * write-friendly format
  * shallow index
- **Data Warehouse**
  - ephemeral subset
  - deeply indexed
  - SQL

---


# Introduction
## Levels of analytics

![bg right](figures/fire.jpg)
- in-app: basic stats, adjust level of details of telemetry
- Basic stats over multiple sessions: MTBF, max memory use
- Deep inspection of a single session: Mem use of every asset, flame graphs
- Aggregates of high-frequency events over many sessions: Heat maps
---

# Legion Performance Analytics
- Introduction
- **Requirements and implications**
- Architecture
- Roadmap

---
# Requirements
## Frequency of events

![bg right](figures/timeline.jpg)

- High Frequency: thousands of events per frame
  * begin/end function call
  * begin/end asset-specific scope
  * memory alloc/free

---
# Requirements
## Frequency of events

![bg right](figures/time.jpg)

- Frame metrics
  * frame time, engine time, render sync
  * player health, #npcs
  * process memory allocated/available
  * i.e. mostly time series
  
---
# Requirements
## Frequency of events
![bg right](figures/controller.jpg)
- Behaviour events
  * begin/end system state
  * gameplay events
  * user input
  
---
# Requirements
## Frequency of events
![bg right](figures/train.jpg)
- Logs
  * begin/end app state (world loaded, in-play, matchmaking)
  * warnings
  * crashes with callstack

---
# Requirements
## Generic and Extensible event stream format

- Open/Closed principle: open for extension, but closed for modification
- Adding a feature-specific event should have no impact on ingestion pipeline
- No magic: specific reports/views depend on the presence of specific events
  * tagging of streams to advertise the purpose/suitability
  * i.e. dynamic duck typing
- not limited to time series

---
# Requirements
## Generic and Extensible event stream format
### Performance characteristics
- write-friendly
  * most work is done with memcpy
  * important size optim: object references
- ingest-friendly
  * store without parsing whole block
  * compressed payload is not decompressed
- generic reader
  * as generic as JSON
  * metadata to decode the writer's memory model
---
# Requirements
## Understanding distributed applications

![bg right](figures/merlin.jpg)
- one application session can extend to multiple processes
- sync clock easier for RPC model of distributed computation

---
# Requirements
## 5 views to rule over all data
### List / Table / Search
![bg right](figures/liste.jpg)

* recent sessions
* top crashes
* cpu budget report
  
---

# Requirements
## 5 views to rule over all data
### Time series

![bg right](figures/time.jpg)

* Individual frame times over time
* Player health over time
* Cohort engagement over 30 days

---
# Requirements
## 5 views to rule over all data
### Graphs & Trees
![bg right](figures/metro.jpg)

* Cumulative function call statistics
* Loaded object graph

---
# Requirements
## 5 views to rule over all data
### Timeline
![bg right](figures/timeline.jpg)
* Call tree instances per thread


---
# Requirements
## 5 views to rule over all data
### Heatmap

![bg right](figures/precision_farming.jpg)

* death map
* geographic slow frames distribution

---
# Requirements
## non-requirements

![bg right](figures/thisisengineering.jpg)

- interactive debugging
- per-pixel profiling
- low-level cpu events (L1 cache miss, branch mispredictions, ...)

## not yet
- Video streaming & overlay
- cpu sampling
- context switches

---
# Legion Performance Analytics
- Introduction
- Requirements
- **Architecture**
- Roadmap

---
# Architecture
## Object hierarchy

![bg right](figures/bibli.jpg)

- Process instance 
  - Stream
    - Stream block
      - Event

---
# Architecture
## Online architecture

![bg right:65%](figures/telemetry_architecture.svg)

Eventually, the analytics app could be hybrid like the editor.

Great bandwidth to read data, native rendering of complex graphs, stream result.

---
# Architecture
## Integration/reuse of existing solutions

Many ideas in common with `tracing` crate from the `tokio` project.
https://docs.rs/tracing/0.1.26/tracing/
But `Collect` trait at the center of the system is a poor fit.

Could support the interface to get visibility into crates that are already instrumented.

![width:400px](figures/tracing.svg)
![width:400px](figures/bevy_logo_dark.svg)


---
# Legion Performance Analytics
- Introduction
- Requirements
- Architecture
- **Roadmap**

---
# Roadmap

![bg right](figures/skellington.png)
<!--- (https://www.pngegg.com/en/png-zocmk)-->

## Halloween

- Initial version of client telemetry library
- Local ingestion server (sqlite & files)
- CLI analytics (csv output)

## Christmas
- Web Analytics client (Vue.js & Canvas)
- Visualization of call tree timeline

---
![bg](figures/question_mark.jpg)
