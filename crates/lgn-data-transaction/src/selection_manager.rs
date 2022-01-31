//! Transaction Operation to Modify the Active Selection

use std::sync::{Arc, RwLock};

use lgn_data_runtime::ResourceTypeAndId;

struct Inner {
    selection: Vec<ResourceTypeAndId>,
    changed: bool,
}

/// Maintain the record of the Active Selections (using inner mutability)
pub struct SelectionManager {
    inner: std::sync::RwLock<Inner>,
}

impl SelectionManager {
    /// Create a new `SelectionManager`
    pub fn create() -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(Inner {
                selection: Vec::new(),
                changed: false,
            }),
        })
    }

    /// Set the active selection (returns old selection)
    pub fn set_selection(&self, new_selection: &[ResourceTypeAndId]) -> Vec<ResourceTypeAndId> {
        let mut inner = self.inner.write().unwrap();
        inner.changed = true;
        std::mem::replace(&mut inner.selection, new_selection.to_vec())
    }

    /// Add new object to the list of selected resources
    pub fn add_to_selection(&self, new_selection: &[ResourceTypeAndId]) {
        let mut inner = self.inner.write().unwrap();
        inner.changed = true;
        inner.selection.extend(new_selection);
    }

    /// Add new object to the list of selected resources
    pub fn toggle_selection(&self, toggle_list: &[ResourceTypeAndId]) {
        let mut inner = self.inner.write().unwrap();
        inner.changed = true;
        for selection in toggle_list {
            if let Some((index, _res)) = inner
                .selection
                .iter()
                .enumerate()
                .find(|(_index, item)| *item == selection)
            {
                inner.selection.remove(index);
            } else {
                inner.selection.push(*selection);
            }
        }
    }

    /// Remove object to the list of selected resources
    pub fn remove_from_selection(&self, new_selection: &[ResourceTypeAndId]) {
        let mut inner = self.inner.write().unwrap();
        inner.changed = true;
        for selection in new_selection {
            if let Some((index, _res)) = inner
                .selection
                .iter()
                .enumerate()
                .find(|(_index, item)| *item == selection)
            {
                inner.selection.remove(index);
            }
        }
    }

    ///  Retrieve the Active Selection set
    pub fn update(&self) -> Option<Vec<ResourceTypeAndId>> {
        let mut inner = self.inner.write().unwrap();
        if inner.changed {
            inner.changed = false;
            Some(inner.selection.clone())
        } else {
            None
        }
    }
}
