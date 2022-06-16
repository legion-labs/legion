use crate::tmp::graph_nodes::GraphValueType;

pub trait EditorGraphNode {
    fn get_value_type() {}

    fn get_allowed_parent_graph_types() {}

    // Compile this node into its runtime representation. Returns the node index of the compiled node.
    fn compile() {}
}

pub struct DataSlotEditorNode {}

impl DataSlotEditorNode {
    fn are_slot_values_valid() -> bool {
        false
    }
}

impl EditorGraphNode for DataSlotEditorNode {}

pub struct ResultEditorNode {
    value_type: GraphValueType,
}

impl EditorGraphNode for ResultEditorNode {
    // fn initialize() {}

    fn compile() {}
}

// Todo! Derive from EditorGraphNode
pub struct ControlParameterEditorNode {
    name: String,
    parameter_category: String,
    graph_value_type: GraphValueType,
}

impl ControlParameterEditorNode {
    fn initialize() {}
}

impl EditorGraphNode for ControlParameterEditorNode {}

pub struct VirtualParameterEditorNode {
    name: String,
    parameter_category: String,
    graph_value_type: GraphValueType,
}

impl VirtualParameterEditorNode {
    pub fn initialize() {}
}

pub struct FlowGraph {
    // graph_type: GraphType,
}

pub struct ParameterReferenceEditorNode {}

impl FlowGraph {
    /* Node factory methods */
    pub fn create_node() {}
}
