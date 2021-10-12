use crate::ScopeDesc;
use transit::prelude::*;

pub type GetScopeDesc = fn() -> ScopeDesc;

pub trait ScopeEvent {
    fn get_time(&self) -> u64;
    fn get_scope(&self) -> GetScopeDesc;
}

#[derive(Debug, TransitReflect)]
pub struct BeginScopeEvent {
    pub time: u64,
    pub scope: fn() -> ScopeDesc,
}

impl InProcSerialize for BeginScopeEvent {}
impl ScopeEvent for BeginScopeEvent {
    fn get_time(&self) -> u64 {
        self.time
    }

    fn get_scope(&self) -> GetScopeDesc {
        self.scope
    }
}

#[derive(Debug, TransitReflect)]
pub struct EndScopeEvent {
    pub time: u64,
    pub scope: fn() -> ScopeDesc,
}

impl InProcSerialize for EndScopeEvent {}

impl ScopeEvent for EndScopeEvent {
    fn get_time(&self) -> u64 {
        self.time
    }

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
