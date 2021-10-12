use std::any::Any;

use legion_data_runtime::{Reference, Resource, ResourceId};

#[derive(Default)]
pub(crate) struct SecondaryAssets(Vec<ResourceId>);

impl SecondaryAssets {
    pub(crate) fn push<T>(&mut self, asset_id: &Reference<T>)
    where
        T: Any + Resource,
    {
        if let Reference::Passive(asset_id) = asset_id {
            self.0.push(*asset_id);
        }
    }
}

impl<'a> Extend<&'a ResourceId> for SecondaryAssets {
    fn extend<T: IntoIterator<Item = &'a ResourceId>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl IntoIterator for SecondaryAssets {
    type Item = ResourceId;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
