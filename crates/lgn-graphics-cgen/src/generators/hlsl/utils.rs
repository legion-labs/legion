use crate::db::{CGenType, Model, NativeType, StructMember};

static FLOAT_TYPE_NAMES: [&str; 4] = ["float", "float2", "float3", "float4"];
static UINT_TYPE_NAMES: [&str; 4] = ["uint", "uint2", "uint3", "uint4"];
static HALF_TYPE_NAMES: [&str; 4] = ["half", "half2", "half3", "half4"];

impl CGenType {
    pub(super) fn to_hlsl_name(&self) -> &str {
        let type_name = match self {
            CGenType::Native(e) => match e {
                NativeType::Float(n) => {
                    assert!(*n >= 1 && *n <= 4);
                    FLOAT_TYPE_NAMES[n - 1]
                }
                NativeType::Uint(n) => {
                    assert!(*n >= 1 && *n <= 4);
                    UINT_TYPE_NAMES[n - 1]
                }
                NativeType::Half(n) => {
                    assert!(*n >= 1 && *n <= 4);
                    HALF_TYPE_NAMES[n - 1]
                }
                NativeType::Float4x4 => "float4x4",
            },
            CGenType::Struct(e) => e.name.as_str(),
            CGenType::BitField(e) => e.name.as_str(),
        };
        type_name
    }
}

pub(super) fn member_declaration(model: &Model, member: &StructMember) -> String {
    let hlsl_name = member.ty_handle.get(model).to_hlsl_name();

    if let Some(array_len) = member.array_len {
        format!("{} {}[{}];", hlsl_name, member.name, array_len)
    } else {
        format!("{} {};", hlsl_name, member.name)
    }
}

pub(super) fn is_matrix_type(model: &Model, member: &StructMember) -> bool {
    match member.ty_handle.get(model) {
        CGenType::Native(native_type) => match native_type {
            NativeType::Float(_) | NativeType::Uint(_) | NativeType::Half(_) => false,
            NativeType::Float4x4 => true,
        },
        CGenType::BitField(_) => false,
        CGenType::Struct(struct_type) => {
            let mut found_matrix = false;
            for m in &struct_type.members {
                if is_matrix_type(model, m) {
                    found_matrix = true;
                }
            }
            found_matrix
        }
    }
}

pub(super) fn load_declaration(model: &Model, member: &StructMember, offset: u32) -> String {
    match member.ty_handle.get(model) {
        CGenType::Native(native_type) => match native_type {
            NativeType::Float(n) => {
                format!(
                    "value.{} = buffer.Load<{}{}>(va + {});",
                    member.name,
                    FLOAT_TYPE_NAMES[n - 1],
                    if let Some(array_len) = member.array_len {
                        format!("[{}]", array_len)
                    } else {
                        String::new()
                    },
                    offset
                )
            }
            NativeType::Uint(n) => {
                format!(
                    "value.{} = buffer.Load<{}{}>(va + {});",
                    member.name,
                    UINT_TYPE_NAMES[n - 1],
                    if let Some(array_len) = member.array_len {
                        format!("[{}]", array_len)
                    } else {
                        String::new()
                    },
                    offset
                )
            }
            NativeType::Half(n) => {
                format!(
                    "value.{} = buffer.Load<{}{}>(va + {});",
                    member.name,
                    HALF_TYPE_NAMES[n - 1],
                    if let Some(array_len) = member.array_len {
                        format!("[{}]", array_len)
                    } else {
                        String::new()
                    },
                    offset
                )
            }
            NativeType::Float4x4 => {
                let array_len = if let Some(array_len) = member.array_len {
                    array_len
                } else {
                    1
                };
                let column_count = array_len * 4;

                let mut unpack = format!(
                    "float4 {}[{}] = buffer.Load<float4[{}]>(va + {});\n\n",
                    member.name, column_count, column_count, offset,
                );

                for array_index in 0..array_len {
                    let build =
                        format!(
                        "        value.{}{} = float4x4(float4({}[0].x, {}[1].x, {}[2].x, {}[3].x),
                               float4({}[0].y, {}[1].y, {}[2].y, {}[3].y),
                               float4({}[0].z, {}[1].z, {}[2].z, {}[3].z),
                               float4({}[0].w, {}[1].w, {}[2].w, {}[3].w));",
                        member.name,
                        if array_len > 1 {
                            format!("[{}]", array_index)
                        }
                        else {String::new()},
                        member.name, member.name, member.name, member.name,
                        member.name, member.name, member.name, member.name,
                        member.name, member.name, member.name, member.name,
                        member.name, member.name, member.name, member.name,
                    );

                    unpack += build.as_str();
                }

                unpack
            }
        },
        CGenType::BitField(_) => {
            format!(
                "value.{}.value = buffer.Load<uint>(va + {});",
                member.name, offset
            )
        }
        CGenType::Struct(s) => {
            format!(
                "value.{}.value = Load{}(buffer, va + {});",
                member.name, s.name, offset
            )
        }
    }
}
