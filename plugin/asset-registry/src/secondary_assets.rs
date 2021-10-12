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

impl<'a, T> Extend<&'a Reference<T>> for SecondaryAssets
where
    T: Any + Resource,
{
    fn extend<I: IntoIterator<Item = &'a Reference<T>>>(&mut self, iter: I) {
        for asset_id in iter {
            self.push(asset_id);
        }
    }
}

impl IntoIterator for SecondaryAssets {
    type Item = ResourceId;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
