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
    UnpackPrefab,
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
    pub serialization_status: SyncStatus,
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
pub enum SyncStatus {
    Unsynced,
    Headless,
    OutofSync,
    Synced,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::Unsynced
    }
}

impl SyncStatus {
    pub fn is_synced_at_all(&self) -> bool {
        match self {
            SyncStatus::Unsynced => false,
            SyncStatus::Headless => false,
            SyncStatus::OutofSync => true,
            SyncStatus::Synced => true,
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
    pub edit_name: Option<String>,
}

impl Default for EntityListInformation {
    fn default() -> Self {
        EntityListInformation {
            open: false,
            color: Color::WHITE,
            edit_name: None,
        }
    }
}
