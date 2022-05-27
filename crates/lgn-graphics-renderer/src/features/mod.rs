mod light_feature;

mod model_feature;
pub use model_feature::*;

use lgn_utils::HashMap;
use std::any::TypeId;

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

pub trait RenderFeature: 'static + Send + Sync {}

pub struct RenderFeaturesBuilder {
    features: HashMap<RenderFeatureId, Box<dyn RenderFeature>>,
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
            features_map,
            features,
        }
    }
}

pub struct RenderFeatures {
    features_map: HashMap<RenderFeatureId, usize>,
    features: Vec<Box<dyn RenderFeature>>,
}

impl RenderFeatures {
    pub fn update(&self) {}
}
