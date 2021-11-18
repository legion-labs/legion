#ifndef TYPE_LAYOUT_B
#define TYPE_LAYOUT_B

	#include "layout_a.hlsl"
	
	struct LayoutB {
		float3 a;
		float4 b;
		LayoutA c;
	}; // LayoutB
	
#endif
