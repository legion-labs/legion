// mod inc;

struct LayoutA {
    a: Float1,
    b: Float2,
}

struct LayoutB {
    a: Float3,
    b: Float4,
    c: LayoutA,
}

// ----------------------------------------

// - Struct:
//     name : LayoutB
//     members :
//       - { type : "Float3", name : "a" }
//       - { type : "Float4", name : "b" }
//       - { name : "c", type : "LayoutA" }

// - DescriptorSet:
//     name : "Shared"
//     frequency : 0
//     descriptors:
//       - { name : "smp", type: "Sampler" }
//       - { name : "a", type: "ConstantBuffer", inner_type : "LayoutA"  }
//       - { name : "b", type: "StructuredBuffer", inner_type : "LayoutB"  }
//       - { name : "brw", type: "RWStructuredBuffer",  inner_type : "LayoutA"  }
//       - { name : "c", type: "ByteAddressBuffer" }
//       - { name : "crw", type: "RWByteAddressBuffer"  }
//       - { name : "d", type: "Texture2D", inner_type : "Float4" }
//       - { name : "drw", type: "RWTexture2D", inner_type : "Float2" }

// - DescriptorSet:
//     name : "Local"
//     frequency : 1
//     descriptors:
//       - { name : "smp2", type: "Sampler" }
//       - { name : "a2", type: "ConstantBuffer", inner_type : "LayoutA"  }
//       - { name : "b2", type: "StructuredBuffer", inner_type : "LayoutA"  }
//       - { name : "brw2", type: "RWStructuredBuffer",  inner_type : "LayoutB"  }
//       - { name : "c2", type: "ByteAddressBuffer" }
//       - { name : "crw2", type: "RWByteAddressBuffer"  }
//       - { name : "d2", type: "Texture2D", inner_type : "Float4" }
//       - { name : "drw2", type: "RWTexture2D", inner_type : "Float1" }

// - PipelineLayout:
//     name : "PL"
//     descriptorssets : [ "Local", "Shared" ]
//     pushconstants :
//       - { name : "a", type : "Float1" }
//       - { name : "b", type : "LayoutA" }
