use super::{Color, ComponentBounds, SerializedEntity};
use uuid::Uuid;

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

#[derive(Debug, PartialEq, Copy, Clone)]
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
    pub fn new<T: ComponentBounds + Clone>(
        comp: &super::Component<T>,
        serialized_entity: Option<&SerializedEntity>,
        draft_mode: bool,
    ) -> SyncStatus {
        serialized_entity
            .map(|se| {
                if comp.is_serialized(se) {
                    SyncStatus::Synced
                } else {
                    SyncStatus::OutofSync
                }
            })
            .unwrap_or_else(|| {
                if draft_mode {
                    SyncStatus::Headless
                } else {
                    SyncStatus::Unsynced
                }
            })
    }

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
pub struct ParentSyncStatus {
    pub serialized: SyncStatus,
    pub prefab: SyncStatus,
}

impl ParentSyncStatus {
    pub fn new<T: ComponentBounds + Clone>(
        comp: &super::Component<T>,
        serialized_entity: Option<&SerializedEntity>,
        prefab_entity: Option<&SerializedEntity>,
        draft_mode: bool,
    ) -> ParentSyncStatus {
        ParentSyncStatus {
            serialized: SyncStatus::new(comp, serialized_entity, draft_mode),
            prefab: SyncStatus::new(comp, prefab_entity, draft_mode),
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

#[derive(Debug, Clone)]
pub enum ComponentInspectorListAction {
    Delete,
    RevertToParentPrefab,
    ComponentPostAction(ComponentSerializationCommandType),
    EntityPostAction(EntitySerializationCommandType),
}

#[derive(Debug, Clone)]
pub enum ComponentInspectorPostAction {
    ComponentCommands(ComponentSerializationCommand),
    EntityCommands(EntitySerializationCommand),
}

#[derive(Debug, Clone)]
pub struct ComponentSerializationCommand {
    pub delta: serde_yaml::Value,
    pub command_type: ComponentSerializationCommandType,
}

#[derive(Debug, Copy, Clone)]
pub enum ComponentSerializationCommandType {
    Serialize,
    StopSerializing,
    Revert,
    ApplyOverrideToParentPrefab,
    RevertToParentPrefab,
}

#[derive(Debug, Copy, Clone)]
pub struct EntitySerializationCommand {
    pub id: Uuid,
    pub command_type: EntitySerializationCommandType,
}

#[derive(Debug, Copy, Clone)]
pub enum EntitySerializationCommandType {
    Revert,
    Overwrite,
    StopSerializing,
}
