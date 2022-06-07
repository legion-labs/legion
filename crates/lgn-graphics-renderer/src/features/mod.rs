mod light_feature;

mod model_feature;
pub use model_feature::*;

use lgn_utils::HashMap;
use std::{any::TypeId, slice::Iter};

use crate::core::{LayerId, RenderListSlice, Requirement, ViewId};

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

pub trait RenderFeature: 'static + Send + Sync {
    fn get_render_list_requirement(
        &self,
        view_id: ViewId,
        layer_id: LayerId,
    ) -> Option<Requirement>;
    fn prepare_render_list(&self, view_id: ViewId, layer_id: LayerId, builder: RenderListSlice);
}

type BoxedRenderFeature = Box<dyn RenderFeature>;

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
            _features: features,
        }
    }
}

pub struct RenderFeatures {
    _features_map: HashMap<RenderFeatureId, usize>,
    _features: Vec<BoxedRenderFeature>,
}

impl RenderFeatures {
    pub fn iter(&self) -> Iter<'_, BoxedRenderFeature> {
        self._features.iter()
    }

    pub fn as_slice(&self) -> &'_ [BoxedRenderFeature] {
        &self._features
    }
}
