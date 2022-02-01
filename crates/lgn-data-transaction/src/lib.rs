//! `Transaction` system

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]
#![warn(missing_docs)]

mod data_manager;
pub use data_manager::*;

mod build_manager;
pub use build_manager::*;

mod transaction;
pub use transaction::Transaction;
pub(crate) use transaction::TransactionOperation;

mod lock_context;
pub use lock_context::LockContext;

pub mod create_resource_operation;
pub use create_resource_operation::*;

pub mod delete_resource_operation;
pub use delete_resource_operation::*;

pub mod rename_resource_operation;
pub use rename_resource_operation::*;

pub mod clone_resource_operation;
pub use clone_resource_operation::*;

pub mod update_property_operation;
pub use update_property_operation::*;

pub mod array_element_operation;
pub use array_element_operation::*;

pub mod reparent_resource_operation;
pub use reparent_resource_operation::*;
#[cfg(test)]
pub(crate) mod test_transaction;
