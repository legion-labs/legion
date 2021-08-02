//! This module defines a test asset.
//!
//! It is used to test the data compilation process until we have a proper asset available.
use crate::AssetType;

/// Type id of test asset.
pub const TYPE_ID: AssetType = AssetType::new(b"test_asset");
