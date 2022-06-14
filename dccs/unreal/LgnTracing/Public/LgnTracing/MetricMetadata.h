#pragma once
//
//  LgnTracing/MetricMetadata.h
//
#include "LgnTracing/MetricEvents.h"

namespace LgnTracing
{
    template<>
    struct GetEventMetadata< IntegerMetricEvent >
    {
        UserDefinedType operator()()
        {
            return UserDefinedType(
                TEXT("IntegerMetricEvent"),
                sizeof(IntegerMetricEvent),
                false,
                {
                    MAKE_UDT_MEMBER_METADATA(IntegerMetricEvent, "desc", Desc, MetricMetadata*, true),
                    MAKE_UDT_MEMBER_METADATA(IntegerMetricEvent, "value", Value, uint64, false),
                    MAKE_UDT_MEMBER_METADATA(IntegerMetricEvent, "time", Timestamp, uint64, false)
                } );
        }
    };

    template<>
    struct GetEventMetadata< FloatMetricEvent >
    {
        UserDefinedType operator()()
        {
            return UserDefinedType(
                TEXT("FloatMetricEvent"),
                sizeof(FloatMetricEvent),
                false,
                {
                    MAKE_UDT_MEMBER_METADATA(FloatMetricEvent, "desc", Desc, MetricMetadata*, true),
                    MAKE_UDT_MEMBER_METADATA(FloatMetricEvent, "value", Value, f64, false),
                    MAKE_UDT_MEMBER_METADATA(FloatMetricEvent, "time", Timestamp, uint64, false)
                } );
        }
    };

    template<>
    struct GetEventMetadata< MetricMetadataDependency >
    {
        UserDefinedType operator()()
        {
            return UserDefinedType(
                TEXT("MetricMetadataDependency"),
                sizeof(MetricMetadataDependency),
                false,
                {
                    MAKE_UDT_MEMBER_METADATA(MetricMetadataDependency, "id", Id, uint64, false),
                    MAKE_UDT_MEMBER_METADATA(MetricMetadataDependency, "lod", Lod, uint8, false),
                    MAKE_UDT_MEMBER_METADATA(MetricMetadataDependency, "name", Name, StaticStringRef, true),
                    MAKE_UDT_MEMBER_METADATA(MetricMetadataDependency, "unit", Unit, StaticStringRef, true),
                    MAKE_UDT_MEMBER_METADATA(MetricMetadataDependency, "target", Target, StaticStringRef, true),
                    MAKE_UDT_MEMBER_METADATA(MetricMetadataDependency, "file", File, StaticStringRef, true),
                    MAKE_UDT_MEMBER_METADATA(MetricMetadataDependency, "line", Line, uint32, false),
                } );
        }
    };
}

