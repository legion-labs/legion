use std::{
    alloc::Layout,
    intrinsics::transmute,
    ptr::NonNull,
    slice::{Iter, IterMut},
};

use bumpalo::{collections::Vec as BumpVec, Bump};
use bumpalo_herd::Herd;
use lgn_tracing::span_scope;

use crate::{
    features::{RenderFeature, RenderFeatures},
    RenderContext,
};

use super::{LayerId, ViewId, VisibilitySet};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FatPointer {
    data: *const (),
    vtable: *const (),
}

#[allow(unsafe_code)]
fn get_vtable<T: Callable>() -> *const () {
    let p = std::ptr::NonNull::<T>::dangling();
    let p_dyn: &dyn Callable = unsafe { p.as_ref() };
    let fat_ptr = unsafe { transmute::<_, FatPointer>(p_dyn) };
    fat_ptr.vtable
}

trait Callable {
    fn call(&self, render_context: &mut RenderContext<'_>);
}

impl Callable for () {
    fn call(&self, _render_context: &mut RenderContext<'_>) {}
}

type DropCallableFn = fn(*mut ());
type OptionalDropCallableFn = Option<DropCallableFn>;

#[allow(unsafe_code)]
fn drop_callable<T: Callable>(data: *mut ()) {
    if std::mem::needs_drop::<T>() {
        unsafe { data.cast::<T>().drop_in_place() }
    }
}

fn get_drop_callable_func<T: Callable>() -> OptionalDropCallableFn {
    if std::mem::needs_drop::<T>() {
        Some(drop_callable::<T> as DropCallableFn)
    } else {
        None
    }
}

#[repr(C)]
struct CallableInfo<T: Callable> {
    vtable: *const (),
    drop_fn: OptionalDropCallableFn,
    data: T,
}

pub struct Requirement {
    render_item_count: usize,
    callable_layout: Layout,
}

#[allow(dead_code)]
impl Requirement {
    fn new<T: Callable>(render_item_count: usize) -> Self {
        Self {
            render_item_count,
            callable_layout: Layout::new::<CallableInfo<T>>(),
        }
    }

    fn callable_aligned_size(&self) -> usize {
        let align = self.callable_layout.align();
        let size = self.callable_layout.size();
        (size + align - 1) & !(align - 1)
    }

    fn callable_array_layout(&self) -> Layout {
        let align = self.callable_layout.align();
        let aligned_size = self.callable_aligned_size();
        Layout::from_size_align(aligned_size * self.render_item_count, align).unwrap()
    }
}

struct FeatureRequirement {
    feature_index: usize,
    requirement: Requirement,
}

struct FeatureLayout {
    feature_index: usize,
    size: usize,
    items_offset: usize,
    callable_infos_offset: usize,
    callable_aligned_size: usize,
}

struct RenderListInfo<'a> {
    view_id: ViewId,
    layer_id: LayerId,
    requirements: BumpVec<'a, FeatureRequirement>,
}

impl<'a> RenderListInfo<'a> {
    fn new(bump: &'a Bump, view_id: ViewId, layer_id: LayerId) -> Self {
        Self {
            view_id,
            layer_id,
            requirements: BumpVec::new_in(bump),
        }
    }

    fn finalize(self, bump: &Bump) -> Option<RenderList<'_>> {
        if self.requirements.is_empty() {
            None
        } else {
            Some(RenderList::new(
                self.view_id,
                self.layer_id,
                RenderListLayout::from_requirements(bump, &self.requirements),
                bump,
            ))
        }
    }
}

const RENDER_LIST_ALIGNMENT: usize = 64;

struct RenderListLayout<'a> {
    item_count: usize,
    total_size: usize,
    feature_layouts: BumpVec<'a, FeatureLayout>,
}

fn align(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

impl<'a> RenderListLayout<'a> {
    fn from_requirements(bump: &'a Bump, requirements: &[FeatureRequirement]) -> Self {
        let mut render_item_count = 0;
        for requirement in requirements {
            render_item_count += requirement.requirement.render_item_count;
        }
        let mut feature_layouts = BumpVec::new_in(bump);
        let mut cur_items_offset = 0;
        let mut cur_callable_infos_offset = Layout::array::<RenderListItem>(render_item_count)
            .unwrap()
            .size();

        for requirement in requirements {
            assert!(requirement.requirement.callable_layout.align() <= RENDER_LIST_ALIGNMENT);

            let callables_layout = requirement.requirement.callable_array_layout();

            cur_callable_infos_offset = align(cur_callable_infos_offset, callables_layout.align());

            let feature_layout = FeatureLayout {
                feature_index: requirement.feature_index,
                size: requirement.requirement.render_item_count,
                items_offset: cur_items_offset,
                callable_infos_offset: cur_callable_infos_offset,
                callable_aligned_size: requirement.requirement.callable_aligned_size(),
            };

            cur_items_offset += Layout::array::<RenderListItem>(feature_layout.size)
                .unwrap()
                .size();

            cur_callable_infos_offset += callables_layout.size();

            feature_layouts.push(feature_layout);
        }

        Self {
            item_count: render_item_count,
            total_size: align(cur_callable_infos_offset, RENDER_LIST_ALIGNMENT),
            feature_layouts,
        }
    }

    fn layout(&self) -> Layout {
        Layout::from_size_align(self.total_size, RENDER_LIST_ALIGNMENT).unwrap()
    }

    fn feature_layouts(&self) -> &[FeatureLayout] {
        &self.feature_layouts
    }
}

struct RenderListItem {
    key: u64,
    info: *const (),
}

impl PartialEq for RenderListItem {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for RenderListItem {}

impl PartialOrd for RenderListItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.key.cmp(&other.key))
    }
}

impl Ord for RenderListItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

pub struct RenderList<'a> {
    view_id: ViewId,
    layer_id: LayerId,
    render_list_layout: RenderListLayout<'a>,
    data_block: NonNull<u8>,
}

#[allow(unsafe_code)]
unsafe impl<'a> Send for RenderList<'a> {}

impl<'a> RenderList<'a> {
    fn new(
        view_id: ViewId,
        layer_id: LayerId,
        render_list_layout: RenderListLayout<'a>,
        bump: &Bump,
    ) -> Self {
        let mem_layout = render_list_layout.layout();
        Self {
            view_id,
            layer_id,
            render_list_layout,
            data_block: bump.alloc_layout(mem_layout),
        }
    }

    #[allow(unsafe_code)]
    fn build(&mut self, features: &[Box<dyn RenderFeature>]) {
        for feature_layout in self.render_list_layout.feature_layouts() {
            let render_items = unsafe {
                self.data_block
                    .as_ptr()
                    .add(feature_layout.items_offset)
                    .cast::<RenderListItem>()
            };
            let callable_infos = unsafe {
                self.data_block
                    .as_ptr()
                    .add(feature_layout.callable_infos_offset)
            };

            let render_list_builder = RenderListSlice {
                size: feature_layout.size,
                render_items,
                callable_infos,
                callable_aligned_size: feature_layout.callable_aligned_size,
            };

            {
                features[feature_layout.feature_index].prepare_render_list(
                    self.view_id,
                    self.layer_id,
                    render_list_builder,
                );
            }
        }
    }

    #[allow(unsafe_code)]
    fn sort(&mut self) {
        let s = unsafe {
            std::slice::from_raw_parts_mut(
                self.data_block.cast::<RenderListItem>().as_ptr(),
                self.render_list_layout.item_count,
            )
        };
        s.sort_unstable();
    }

    #[allow(unsafe_code)]
    fn execute(&self, render_context: &mut RenderContext<'_>) {
        for render_item in self.items() {
            let callable_info = unsafe { &*render_item.info.cast::<CallableInfo<()>>() };
            let data_ptr = &callable_info.data as *const ();

            let fat_ptr = FatPointer {
                data: data_ptr,
                vtable: callable_info.vtable,
            };
            let callable: &dyn Callable = unsafe { transmute(fat_ptr) };
            callable.call(render_context);

            if let Some(drop_fn) = callable_info.drop_fn {
                (drop_fn)(data_ptr as *mut ());
            }
        }
    }

    #[allow(unsafe_code)]
    fn items(&self) -> &[RenderListItem] {
        unsafe {
            std::slice::from_raw_parts(
                self.data_block.cast::<RenderListItem>().as_ptr(),
                self.render_list_layout.item_count,
            )
        }
    }
}

#[allow(dead_code)]
pub struct RenderListSlice {
    size: usize,
    render_items: *mut RenderListItem,
    callable_infos: *mut u8,
    callable_aligned_size: usize,
}

#[allow(dead_code)]
impl RenderListSlice {
    fn size(&self) -> usize {
        self.size
    }

    #[allow(unsafe_code)]
    fn write<T: Callable>(&self, index: usize, key: u64, data: T) {
        assert!(index < self.size);

        unsafe {
            let callable = self.callable_infos.cast::<CallableInfo<T>>().add(index);
            let render_item = self.render_items.add(index * self.callable_aligned_size);
            let vtable = get_vtable::<T>();
            let drop_fn = get_drop_callable_func::<T>();

            callable.write(CallableInfo::<T> {
                vtable,
                drop_fn,
                data,
            });

            render_item.write(RenderListItem {
                key,
                info: callable.cast::<()>(),
            });
        }
    }
}

#[allow(dead_code)]
struct RenderListSliceTyped<T: Callable> {
    size: usize,
    render_items: *mut RenderListItem,
    callable_infos: *mut CallableInfo<T>,
    vtable: *const (),
    drop_fn: OptionalDropCallableFn,
}

#[allow(dead_code)]
impl<T> RenderListSliceTyped<T>
where
    T: Callable,
{
    fn new(render_list_slice: RenderListSlice) -> Self {
        let vtable = get_vtable::<T>();
        let drop_fn = get_drop_callable_func::<T>();

        Self {
            size: render_list_slice.size,
            render_items: render_list_slice.render_items,
            callable_infos: render_list_slice.callable_infos.cast::<CallableInfo<T>>(),
            vtable,
            drop_fn,
        }
    }

    fn size(&self) -> usize {
        self.size
    }

    #[allow(unsafe_code)]
    fn write(&self, index: usize, key: u64, data: T) {
        assert!(index < self.size);

        unsafe {
            let callable_info_ptr = self.callable_infos.add(index);

            callable_info_ptr.write(CallableInfo::<T> {
                vtable: self.vtable,
                drop_fn: self.drop_fn,
                data,
            });

            let render_item = self.render_items.add(index);
            render_item.write(RenderListItem {
                key,
                info: callable_info_ptr.cast::<()>(),
            });
        }
    }

    fn iter(self) -> RenderListItemWriterIter<T> {
        RenderListItemWriterIter::<T>::new(self)
    }
}

struct RenderListItemWriterIter<T: Callable> {
    index: usize,
    size: usize,
    render_items: *mut RenderListItem,
    callable_infos: *mut CallableInfo<T>,
    vtable: *const (),
    drop_fn: OptionalDropCallableFn,
}

impl<T> Iterator for RenderListItemWriterIter<T>
where
    T: Callable,
{
    type Item = RenderListItemWriter<T>;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            let render_item = unsafe { self.render_items.add(self.index) };
            let callable_info = unsafe { self.callable_infos.add(self.index) };
            self.index += 1;

            Some(RenderListItemWriter::<T> {
                render_item,
                callable_info,
                vtable: self.vtable,
                drop_fn: self.drop_fn,
            })
        } else {
            None
        }
    }
}

impl<T> RenderListItemWriterIter<T>
where
    T: Callable,
{
    fn new(writer: RenderListSliceTyped<T>) -> Self {
        Self {
            index: 0,
            size: writer.size,
            render_items: writer.render_items,
            callable_infos: writer.callable_infos,
            vtable: writer.vtable,
            drop_fn: writer.drop_fn,
        }
    }
}

struct RenderListItemWriter<T: Callable> {
    render_item: *mut RenderListItem,
    callable_info: *mut CallableInfo<T>,
    vtable: *const (),
    drop_fn: OptionalDropCallableFn,
}

impl<T: Callable> RenderListItemWriter<T> {
    #[allow(unsafe_code)]
    fn write(&self, key: u64, data: T) {
        unsafe {
            let callable_info = self.callable_info;

            callable_info.write(CallableInfo::<T> {
                vtable: self.vtable,
                drop_fn: self.drop_fn,
                data,
            });

            self.render_item.write(RenderListItem {
                key,
                info: callable_info.cast::<()>(),
            });
        }
    }
}

pub struct RenderListSet<'a> {
    render_lists: &'a mut [RenderList<'a>],
    // render_lists: BumpVec<'a, RenderList<'a>>,
}

impl<'a> RenderListSet<'a> {
    fn new(bump: &'a Bump, mut render_list_infos: BumpVec<'_, RenderListInfo<'_>>) -> Self {
        let mut render_lists = BumpVec::new_in(bump);
        for render_list_info in render_list_infos.drain(..) {
            if let Some(render_list) = render_list_info.finalize(bump) {
                render_lists.push(render_list);
            }
        }
        let render_lists = render_lists.into_bump_slice_mut();

        Self { render_lists }
    }

    pub fn as_slice(&self) -> &[RenderList<'a>] {
        // self.render_lists.as_slice()
        self.render_lists
    }

    pub fn as_mut_slice(&mut self) -> &mut [RenderList<'a>] {
        // self.render_lists.as_mut_slice()
        self.render_lists
    }

    fn iter<'r>(&'r self) -> Iter<'_, RenderList<'a>> {
        self.render_lists.iter()
    }

    fn iter_mut<'r>(&'r mut self) -> IterMut<'_, RenderList<'a>> {
        self.render_lists.iter_mut()
    }
}

impl<'r, 'b> IntoIterator for &'r RenderListSet<'b>
where
    'b: 'r,
{
    type Item = &'r RenderList<'b>;

    type IntoIter = Iter<'r, RenderList<'b>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, 'b> IntoIterator for &'r mut RenderListSet<'b>
where
    'b: 'r,
{
    type Item = &'r mut RenderList<'b>;

    type IntoIter = IterMut<'r, RenderList<'b>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct PrepareRenderContext<'rt> {
    pub herd: &'rt Herd,
    pub bump: &'rt Bump,
    pub visibility_set: &'rt VisibilitySet<'rt>,
    pub features: &'rt RenderFeatures,
}

impl<'rt> PrepareRenderContext<'rt> {
    #[must_use]
    pub fn execute(&self) -> &'rt RenderListSet<'rt> {
        span_scope!("PrepareRender");

        let bump = self.bump;

        let render_list_infos = {
            span_scope!("get_render_list_requirements");
            let mut render_list_infos = BumpVec::new_in(bump);
            for viz_view in self.visibility_set.views() {
                for layer_id in &viz_view.layers {
                    let view_id = viz_view.id;
                    let layer_id = *layer_id;
                    let mut builder = RenderListInfo::new(bump, view_id, layer_id);
                    for (feature_index, feature) in self.features.iter().enumerate() {
                        if let Some(requirement) =
                            feature.get_render_list_requirement(view_id, layer_id)
                        {
                            builder.requirements.push(FeatureRequirement {
                                feature_index,
                                requirement,
                            });
                        }
                    }
                    render_list_infos.push(builder);
                }
            }
            render_list_infos
        };

        let render_list_set = bump.alloc(RenderListSet::new(bump, render_list_infos));

        {
            span_scope!("build and sort renderlist");

            let features = self.features.as_slice();

            for render_list in render_list_set.as_mut_slice() {
                {
                    span_scope!("build renderlist");
                    render_list.build(features);
                }
                {
                    span_scope!("sort renderlist");
                    render_list.sort();
                }
            }
        }

        render_list_set
    }
}
