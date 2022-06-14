#pragma once
//
//  LgnTracing/Dispatch.h
//
#include <string>
#include <memory>
#include <mutex>
#include <vector>
#include "HAL/Platform.h"
#include "LgnTracing/Fwd.h"
class FScopeLock;

namespace LgnTracing
{
    typedef std::wstring (*NewGuid)();
    typedef void (*ThreadStreamCallback)( ThreadStream* );

    
    class LGNTRACING_API Dispatch
    {
    public:
        static void Init(NewGuid allocNewGuid,
                         const ProcessInfoPtr& processInfo,
                         const std::shared_ptr<EventSink>& sink,
                         size_t logBufferSize,
                         size_t metricBufferSize,
                         size_t threadBufferSize);
        ~Dispatch();

        friend LGNTRACING_API void Shutdown();
        friend LGNTRACING_API void FlushLogStream();
        friend LGNTRACING_API void FlushMetricStream();
        friend LGNTRACING_API void LogInterop( const LogStringInteropEvent& event );
        friend LGNTRACING_API void LogStaticStr( const LogStaticStrEvent& event );
        friend LGNTRACING_API void IntMetric( const IntegerMetricEvent& event );
        friend LGNTRACING_API void FloatMetric( const FloatMetricEvent& event );
        friend LGNTRACING_API void BeginScope( const BeginThreadSpanEvent& event );
        friend LGNTRACING_API void EndScope( const EndThreadSpanEvent& event );

        friend LGNTRACING_API void ForEachThreadStream( ThreadStreamCallback callback );
        
        template< typename T >
        friend void QueueLogEntry( const T& event );

        template< typename T >
        friend void QueueMetric( const T& event );

        template< typename T >
        friend void QueueThreadEvent( const T& event );
        
        friend ThreadStream* GetCurrentThreadStream();
        
    private:
        Dispatch(NewGuid allocNewGuid,
                 const std::shared_ptr<EventSink>& sink,
                 const ProcessInfoPtr& processInfo,
                 size_t logBufferSize,
                 size_t metricBufferSize,
                 size_t threadBufferSize);

        typedef std::unique_ptr<std::lock_guard<std::recursive_mutex>> GuardPtr;
        void FlushLogStreamImpl(GuardPtr& guard);
        void FlushMetricStreamImpl(GuardPtr& guard);
        void FlushThreadStream(ThreadStream* stream);
        ThreadStream* AllocThreadStream();
        void PublishThreadStream(ThreadStream* stream);

        NewGuid AllocNewGuid;
    
        std::shared_ptr<EventSink> Sink;
        ProcessInfoPtr CurrentProcessInfo;

        std::recursive_mutex LogMutex;
        std::shared_ptr<LogStream> LogEntries;
        size_t LogBufferSize;

        std::recursive_mutex MetricMutex;
        std::shared_ptr<MetricStream> Metrics;
        size_t MetricBufferSize;

        std::recursive_mutex ThreadStreamsMutex;
        std::vector<ThreadStream*> ThreadStreams;
        size_t ThreadBufferSize;
    };

    extern LGNTRACING_API Dispatch* GDispatch;
} // namespace
