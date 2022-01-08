use lgn_transit::prelude::*;

use crate::ScopeDesc;

#[derive(Debug, TransitReflect)]
pub struct BeginScopeEvent {
    pub time: i64,
    pub scope: &'static ScopeDesc, /* can't be GetScopeDesc because the reflection would not see
                                    * it as a reference */
}

impl InProcSerialize for BeginScopeEvent {}

#[derive(Debug, TransitReflect)]
pub struct EndScopeEvent {
    pub time: i64,
    pub scope: &'static ScopeDesc, /* can't be GetScopeDesc because the reflection would not see
                                    * it as a reference */
}

impl InProcSerialize for EndScopeEvent {}

#[derive(Debug, TransitReflect)]
pub struct ReferencedScope {
    pub id: u64,
    pub name: *const u8,
    pub filename: *const u8,
    pub line: u32,
}

impl InProcSerialize for ReferencedScope {}
