---
marp: true
---

![bg](figures/health.jpg)
![](white)
# Legion Performance Analytics

May 2022

---
# Legion Performance Analytics
## Introduction

![bg right vertical width:500px height:200px](figures/uelogs.png)
![bg width:500px height:200px](figures/editor_metrics.png)
![bg width:500px height:200px](figures/editor_timeline_color.png)
- Logs, metrics, traces
- Latency, stability, satisfaction
- For local, distributed & cloud native applications

---
# Legion Performance Analytics
## Full stack solution

- One protocol
  * Structured events
  * time series are not general enough
- One data lake
  * no loose files
- One UI
  * High performance web user interface
  * Tight integration between the logs, metrics & traces

---
# Legion Performance Analytics
## Designed for scale

- High event density: 40 ns overhead / event
  * platform-specific memory layout
  * the protocol is the memory layout
- Store everything: event batch stored in S3
  * shallow metadata index in SQL database
- Distributed ingestion
  * optimized & stateless

---
# Legion Performance Analytics
## Analytics is the new profiling

- Designed for iterating in development
  * data available as soon as it is sent
  * no nightly indexing job
- Faster reaction time
  * know the bugs *before* they become problems
- Understand the issue without reproducing it
- Spend time on the worst issues, not the first one you find

---
# Status Quo

Data | Development | Live
--------- | --------- | ---------
Client Logs | loose files | ELK/sampling
Client Metrics | debug overlay | low frequency
Client Traces | interactive profiling | :x:
Server Logs | loose files | ELK 
Server Metrics | :x: | Prometheus, Grafana
Server Traces  | loose files | :x:

---
# Comparison

Capability | Legion Performance Analytics | Splunk/Datadog | Logz | RAD Telemetry
---------- | ---------------------------- | -------------- | ---- | -------------
Logs       | :heavy_check_mark:           | :heavy_check_mark:  | :heavy_check_mark: | :x:
Time series| :heavy_check_mark:           | :heavy_check_mark:  | :heavy_check_mark: | :heavy_check_mark:
Traces     | :heavy_check_mark:           | :heavy_check_mark:  | :x: | :heavy_check_mark:
$calable   | :green_heart:                | :x:  | :heavy_check_mark: | :no_entry:

---
# Architecture

![height:600px](figures/telemetry_architecture.svg)

---
![bg](figures/question_mark.jpg)
