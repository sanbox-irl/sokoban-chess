use super::Color;

#[derive(Debug, PartialEq, Eq)]
pub enum NameRequestedAction {
    ChangeName(String),

    Serialize,
    Unserialize,
    ToggleInspect,

    Clone,
    Delete,

    PromoteToPrefab,
    GoToPrefab,

    LogEntity,
    LogSerializedEntity,
    LogPrefab,
}

pub struct NameInspectorResult {
    pub show_children: bool,
    pub requested_action: Option<NameRequestedAction>,
}

impl Default for NameInspectorResult {
    fn default() -> Self {
        Self {
            show_children: true,
            requested_action: None,
        }
    }
}

impl NameInspectorResult {
    pub fn new() -> Self {
        let mut me: Self = Default::default();
        me.show_children = true;

        me
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct NameInspectorParameters {
    pub has_children: bool,
    pub depth: usize,
    pub prefab_status: PrefabStatus,
    pub being_inspected: bool,
    pub is_serialized: bool,
}

impl NameInspectorParameters {
    pub fn with_scene_graph_data(has_children: bool, depth: usize) -> Self {
        Self {
            has_children,
            depth,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrefabStatus {
    None,
    Prefab,
    PrefabInstance,
}

impl Default for PrefabStatus {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityListInformation {
    pub open: bool,
    pub color: Color,
    pub edit_name: NameEdit,
}

impl Default for EntityListInformation {
    fn default() -> Self {
        EntityListInformation {
            open: false,
            color: Color::WHITE,
            edit_name: NameEdit::NoEdit,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NameEdit {
    First,
    Editing,
    NoEdit,
}
