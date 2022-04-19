/// The source resource is a trait that exists only in offline mode, in the editor.
#[async_trait::async_trait]
pub trait SourceResource: lgn_data_runtime::Resource {
    /// Return a new named Resource
    fn new_named(name: &str) -> Self
    where
        Self: Sized + Default,
    {
        let mut value = Self::default();
        get_meta_mut(&mut value).name = name.into();
        value
    }
}

/// Returns the metadata embedded in the Resource.
pub fn get_meta(res: &dyn lgn_data_runtime::Resource) -> &crate::offline::Metadata {
    let meta_ptr = lgn_data_model::utils::find_property(res.as_reflect(), "meta").unwrap();
    unsafe { &*(meta_ptr.base.cast::<crate::offline::Metadata>()) }
}

/// Returns a mutable version of the metadata embedded in the Resource.
pub fn get_meta_mut(res: &mut dyn lgn_data_runtime::Resource) -> &mut crate::offline::Metadata {
    let meta_ptr = lgn_data_model::utils::find_property_mut(res.as_reflect_mut(), "meta").unwrap();
    unsafe { &mut *(meta_ptr.base.cast::<crate::offline::Metadata>()) }
}
