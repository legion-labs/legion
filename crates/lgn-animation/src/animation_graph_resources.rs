#![allow(dead_code)]

use crate::animation_skeleton::Skeleton;

pub struct GraphDataSet {
    variation_id: i32,
    skeleton: Skeleton,
    // resources: Vec<Resource>,
}

impl GraphDataSet {
    fn is_valid() {
        /* */
    }
}

pub struct GraphDefinition {
    persistent_node_indices: Vec<i16>,
    // instance_node_start_offsets: Vec<u32>,
    // instance_required_memory: u32,
    // instance_required_alignment: u32,
    // num_control_parameters: i32,
    root_node_idx: i16,
    // control_parameter_ids: Vec<str>,
    // node_paths: Vec<str>,
    // node_settings: Vec<Settings>,
}

impl GraphDefinition {
    fn is_valid() {
        /* */
    }
}

pub struct GraphVariation {
    graph_definition: GraphDefinition,
    data_set: GraphDataSet,
}

impl GraphVariation {
    fn is_valid() {
        /* */
    }

    #[inline]
    fn get_skeleton() {
        /* */
    }
}
