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

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

    pub fn imgui_color(&self, scene_mode: super::SceneMode) -> [f32; 4] {
        match self {
            SyncStatus::Unsynced => {
                if scene_mode == super::SceneMode::Draft {
                    super::imgui_system::red_warning_color()
                } else {
                    Color::WHITE.into()
                }
            }
            SyncStatus::Headless => super::imgui_system::red_warning_color(),
            SyncStatus::OutofSync => super::imgui_system::yellow_warning_color(),
            SyncStatus::Synced => Color::WHITE.into(),
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
    pub color: [f32; 4],
    pub edit_name: Option<String>,
}

impl Default for EntityListInformation {
    fn default() -> Self {
        EntityListInformation {
            open: false,
            color: Color::WHITE.into(),
            edit_name: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ComponentInspectorListAction {
    Delete,
    RevertSerialization,
    RevertToParentPrefab,
    ComponentInspectorPostAction(ComponentInspectorPostAction),
}

#[derive(Debug, Copy, Clone)]
pub enum ComponentInspectorPostAction {
    Serialize,
    StopSerializing,
    ApplyOverrideToParentPrefab,
}
