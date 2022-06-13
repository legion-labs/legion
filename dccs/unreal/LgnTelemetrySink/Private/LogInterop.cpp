//
//  LgnTelemetrySink/LogInterop.cpp
//
#include "LgnTelemetrySink/LogInterop.h"
#include "LgnTracing/LogEvents.h"
#include "LgnTracing/Dispatch.h"

using namespace LgnTracing;

struct LogBridge : public FOutputDevice
{
    virtual void Serialize( const TCHAR* V, ELogVerbosity::Type Verbosity, const FName& Category ){
        LogLevel::Type level = LogLevel::Invalid;
        switch (Verbosity)
        {
            case ELogVerbosity::Fatal:
            case ELogVerbosity::Error:
                level = LogLevel::Error;
                break;
            case ELogVerbosity::Warning:
                level = LogLevel::Warn;
                break;
            case ELogVerbosity::Display:
                level = LogLevel::Info;
                break;
            case ELogVerbosity::Log:
                level = LogLevel::Debug;
                break;
            default:
                level = LogLevel::Trace;
                        
        };
        LogInterop( LogStringInteropEvent( FPlatformTime::Cycles64(),
                                           level,
                                           StaticStringRef( Category.GetDisplayNameEntry() ),
                                           DynamicString(V) ) );
    }
};

void InitLogInterop()
{
    check(GLog);
    static LogBridge bridge;
    GLog->AddOutputDevice(&bridge);
}

