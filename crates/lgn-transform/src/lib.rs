//! Legion Transform
//!
//! TODO: write documentation.
#![doc = include_str!("../README.md")]

// crate-specific lint exceptions:
//#![allow()]

/// The basic components of the transform crate
pub mod components;
mod systems;
pub use crate::systems::transform_propagate_system;

#[doc(hidden)]
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{components::*, TransformBundle, TransformPlugin};
}

use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_hierarchy::HierarchySystem;
use prelude::{GlobalTransform, Transform};

/// A [`Bundle`] of the [`Transform`] and [`GlobalTransform`]
/// [`Component`](lgn_ecs::component::Component)s, which describe the position of an entity.
///
/// * To place or move an entity, you should set its [`Transform`].
/// * To get the global position of an entity, you should get its [`GlobalTransform`].
/// * For transform hierarchies to work correctly, you must have both a [`Transform`] and a [`GlobalTransform`].
///   * You may use the [`TransformBundle`] to guarantee this.
///
/// ## [`Transform`] and [`GlobalTransform`]
///
/// [`Transform`] is the position of an entity relative to its parent position, or the reference
/// frame if it doesn't have a parent.
///
/// [`GlobalTransform`] is the position of an entity relative to the reference frame.
///
/// [`GlobalTransform`] is updated from [`Transform`] in the system
/// [`transform_propagate_system`].
///
/// This system runs in stage [`CoreStage::PostUpdate`](crate::CoreStage::PostUpdate). If you
/// update the[`Transform`] of an entity in this stage or after, you will notice a 1 frame lag
/// before the [`GlobalTransform`] is updated.
#[derive(Bundle, Clone, Copy, Debug, Default)]
pub struct TransformBundle {
    /// The transform of the entity.
    pub local: Transform,
    /// The global transform of the entity.
    pub global: GlobalTransform,
}

impl TransformBundle {
    /// Creates a new [`TransformBundle`] from a [`Transform`].
    ///
    /// This initializes [`GlobalTransform`] as identity, to be updated later by the
    /// [`CoreStage::PostUpdate`](crate::CoreStage::PostUpdate) stage.
    #[inline]
    pub const fn from_transform(transform: Transform) -> Self {
        Self {
            local: transform,
            // Note: `..Default::default()` cannot be used here, because it isn't const
            ..Self::identity()
        }
    }

    /// Creates a new identity [`TransformBundle`], with no translation, rotation, and a scale of 1
    /// on all axes.
    #[inline]
    pub const fn identity() -> Self {
        Self {
            local: Transform::identity(),
            global: GlobalTransform::identity(),
        }
    }
}

impl From<Transform> for TransformBundle {
    #[inline]
    fn from(transform: Transform) -> Self {
        Self::from_transform(transform)
    }
}

/// Label enum for the systems relating to transform propagation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum TransformSystem {
    /// Propagates changes in transform to childrens' [`GlobalTransform`](crate::components::GlobalTransform)
    TransformPropagate,
}

/// The base plugin for handling [`Transform`] components
#[derive(Default)]
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app
            // Adding these to startup ensures the first update is "correct"
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                systems::transform_propagate_system
                    .label(TransformSystem::TransformPropagate)
                    .after(HierarchySystem::ParentUpdate),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                systems::transform_propagate_system
                    .label(TransformSystem::TransformPropagate)
                    .after(HierarchySystem::ParentUpdate),
            );
    }
}
