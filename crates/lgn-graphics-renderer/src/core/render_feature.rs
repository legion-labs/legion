use lgn_utils::HashMap;
use std::{any::TypeId, slice::Iter};

use crate::core::{RenderListSlice, RenderListSliceRequirement};

use super::{RenderLayerId, VisibleView};

#[derive(PartialEq, Eq, Hash)]
struct RenderFeatureId(TypeId);

impl RenderFeatureId {
    fn new<T>() -> Self
    where
        T: 'static,
    {
        Self(TypeId::of::<T>())
    }
}

pub trait RenderFeature: 'static + Send {
    fn get_render_list_requirement(
        &self,
        _visible_view: &VisibleView,
        _render_layer_id: RenderLayerId,
    ) -> Option<RenderListSliceRequirement> {
        None
    }
    fn prepare_render_list(
        &self,
        _visible_view: &VisibleView,
        _layer_id: RenderLayerId,
        _render_list_slice: RenderListSlice,
    ) {
        unreachable!();
    }
}

pub type BoxedRenderFeature = Box<dyn RenderFeature>;

pub struct RenderFeaturesBuilder {
    features: HashMap<RenderFeatureId, BoxedRenderFeature>,
}

impl RenderFeaturesBuilder {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }

    #[must_use]
    pub fn insert<T>(mut self, feature: T) -> Self
    where
        T: RenderFeature,
    {
        let id = RenderFeatureId::new::<T>();
        self.features.insert(id, Box::new(feature));
        self
    }

    pub fn finalize(mut self) -> RenderFeatures {
        let mut features_map = HashMap::new();
        let mut features = Vec::new();

        for (index, (id, feature)) in self.features.drain().enumerate() {
            features_map.insert(id, index);
            features.push(feature);
        }

        RenderFeatures {
            _features_map: features_map,
            features,
        }
    }
}

pub struct RenderFeatures {
    _features_map: HashMap<RenderFeatureId, usize>,
    features: Vec<BoxedRenderFeature>,
}

impl RenderFeatures {
    pub fn iter(&self) -> Iter<'_, BoxedRenderFeature> {
        self.features.iter()
    }

    pub fn as_slice(&self) -> &'_ [BoxedRenderFeature] {
        &self.features
    }
}
