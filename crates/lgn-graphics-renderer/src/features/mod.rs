mod light_feature;

mod model_feature;
use bumpalo::Bump;
use bumpalo_herd::Herd;
pub use model_feature::*;

use lgn_utils::HashMap;
use smallvec::SmallVec;
use std::{alloc::Layout, any::TypeId, ptr::NonNull};

use crate::gpu_renderer::{RenderLayerId, RenderLayerMask};

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
        _: &PrepareRenderListContext<'_>,
        _: &VisibleView,
        _: RenderLayerId,
    ) -> Option<RenderListRequirement> {
        None
    }
}

pub struct VisibilitySet {}

pub struct VisibleView {}

impl VisibleView {
    fn layers(&self) -> RenderLayerMask {
        RenderLayerMask(0b111)
    }
}

pub struct SortKey {
    value: u64,
}

pub struct RenderItem {
    sort_key: SortKey,
    data: *mut u8,
    render_fun: fn(*mut u8),
}

pub struct RenderList<'a> {
    visible_view: &'a VisibleView,
    render_layer_id: RenderLayerId,
    render_item_count: usize,
    render_items: std::ptr::NonNull<u8>,
}

pub struct PrepareRenderListContext<'a> {
    pub visible_views: &'a [VisibleView],
}

pub struct RenderListRequirement {
    render_item_count: usize,
    attached_data_size: usize,
}

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

struct RenderListFeatureInfo {
    feature_id: usize,
    requirement: RenderListRequirement,
}
struct RenderListInfo {
    view_id: usize,
    layer_id: u32,
    begin_index: usize,
    end_index: usize,
}

impl RenderFeatures {
    pub fn prepare_render<'a>(&self, herd: &Herd, context: &PrepareRenderListContext<'_>) {
        let mut render_list_feature_infos = SmallVec::<[RenderListFeatureInfo; 128]>::new();
        let mut render_list_infos = SmallVec::<[RenderListInfo; 128]>::new();
        let mut begin_index = 0;
        let mut end_index = 0;

        for (visible_view_id, visible_view) in context.visible_views.iter().enumerate() {
            let render_layer_mask = visible_view.layers();
            for render_layer_id in render_layer_mask.iter() {
                for (feature_id, feature) in self.features.iter().enumerate() {
                    let requirement =
                        feature.get_render_list_requirement(context, visible_view, render_layer_id);
                    if let Some(requirement) = requirement {
                        render_list_feature_infos.push(RenderListFeatureInfo {
                            feature_id,
                            requirement,
                        });
                        end_index += 1;
                    }
                }
                if !render_list_feature_infos.is_empty() {
                    render_list_infos.push(RenderListInfo {
                        view_id: visible_view_id,
                        layer_id: render_layer_id,
                        begin_index,
                        end_index,
                    });
                }
                begin_index = end_index;
            }
        }

        for render_list_info in &render_list_infos {
            let r = (render_list_info.begin_index..render_list_info.end_index);
            let mut render_item_count = 0;
            let mut render_item_data_size = 0;
            for render_list_feature_info in &render_list_feature_infos[r] {
                render_item_count += render_list_feature_info.requirement.render_item_count;
                render_item_data_size += render_list_feature_info.requirement.attached_data_size;
            }
        }

        // let bump = herd.get();
        // let mut all_render_list = SmallVec::<[RenderList<'_>; 16]>::new();
        // let mut all_task_list = SmallVec::<[PrepareRenderListTask; 16]>::new();
        // for visible_view in context.visible_views {
        //     let render_layer_mask = visible_view.layers();
        //     for render_layer_id in render_layer_mask.iter() {
        //         let mut layer_task_list = SmallVec::<[PrepareRenderListTask; 16]>::new();
        //         for feature in &self.features {
        //             let task = feature.prepare_render_list(context, visible_view, render_layer_id);
        //             if let Some(task) = task {
        //                 layer_task_list.push(task);
        //             }
        //         }
        //         if !layer_task_list.is_empty() {
        //             let mut render_item_count = 0;
        //             for task in &layer_task_list {
        //                 render_item_count += task.render_item_count;
        //             }
        //             let render_list_size = render_item_count * std::mem::size_of::<RenderItem>();

        //             let render_items =
        //                 bump.alloc_layout(Layout::from_size_align(render_list_size, 64).unwrap());

        //             all_render_list.push(RenderList {
        //                 visible_view,
        //                 render_layer_id,
        //                 render_item_count,
        //                 render_items,
        //             });
        //             all_task_list.append(&mut layer_task_list);
        //         }
        //     }
        // }

        // for task in &all_task_list {
        //     task.fill_render_list_func();
        // }
    }
}
