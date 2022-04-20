use std::{collections::HashMap, sync::RwLock};

use once_cell::sync::OnceCell;

use crate::{spans::SpanMetadata, Verbosity};

static META_DATA_MAP: OnceCell<RwLock<HashMap<&'static str, Box<SpanMetadata>>>> = OnceCell::new();

pub fn lookup_span_metadata(name: &'static str) -> &'static SpanMetadata {
    let meta_data_map = META_DATA_MAP.get_or_init(|| RwLock::new(HashMap::new()));

    if !meta_data_map.read().unwrap().contains_key(name) {
        meta_data_map.write().unwrap().insert(
            name,
            Box::new(SpanMetadata {
                lod: Verbosity::Max,
                name,
                target: module_path!(),
                module_path: module_path!(),
                file: file!(),
                line: line!(),
            }),
        );
    };

    let meta_data_ptr: *const SpanMetadata =
        meta_data_map.read().unwrap().get(name).unwrap().as_ref();

    unsafe { &*meta_data_ptr }
}
