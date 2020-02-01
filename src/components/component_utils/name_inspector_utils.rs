pub struct NameInspectorResult {
    pub reserialize: bool,
    pub inspect: bool,
    pub show_children: bool,
    pub clone: bool,
    pub delete: bool,
}

impl Default for NameInspectorResult {
    fn default() -> Self {
        Self {
            show_children: true,

            reserialize: false,
            inspect: false,
            clone: false,
            delete: false,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct NameInspectorParameters {
    pub has_children: bool,
    pub is_prefab: bool,
    pub depth: usize,
    pub being_inspected: bool,
}
