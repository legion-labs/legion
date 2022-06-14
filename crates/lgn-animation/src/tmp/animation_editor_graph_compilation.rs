// Derive from GraphDefinition
pub struct EditorGraphCompilationContext {}

impl EditorGraphCompilationContext {
    // General compilation

    // Try to get the runtime settings for a node in the graph
    pub fn get_settings() {}

    // This will return an index that can be used to look up the data resource at runtime
    pub fn register_slot_node() {}

    /* State machine compilation */

    // Start compilation of a transition conduit
    pub fn begin_conduit_compilation() {}

    // End compilation of a transition conduit
    pub fn end_conduit_compilation() {}

    // Start compilation of a transition condition conduit
    pub fn begin_transition_conditions_compilation() {}

    /* Rendue la! */
}
