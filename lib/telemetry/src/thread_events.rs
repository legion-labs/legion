use lgn_transit::prelude::*;

use crate::ScopeDesc;

pub type GetScopeDesc = fn() -> ScopeDesc;

pub trait ScopeEvent {
    fn get_scope(&self) -> GetScopeDesc;
}

#[derive(Debug, TransitReflect)]
pub struct BeginScopeEvent {
    pub time: i64,
    pub scope: GetScopeDesc,
}

impl InProcSerialize for BeginScopeEvent {}
impl ScopeEvent for BeginScopeEvent {
    fn get_scope(&self) -> GetScopeDesc {
        self.scope
    }
}

#[derive(Debug, TransitReflect)]
pub struct EndScopeEvent {
    pub time: i64,
    pub scope: GetScopeDesc,
}

impl InProcSerialize for EndScopeEvent {}

impl ScopeEvent for EndScopeEvent {
    fn get_scope(&self) -> GetScopeDesc {
        self.scope
    }
}

#[derive(Debug, TransitReflect)]
pub struct ReferencedScope {
    pub id: u64,
    pub name: *const u8,
    pub filename: *const u8,
    pub line: u32,
}

impl InProcSerialize for ReferencedScope {}
