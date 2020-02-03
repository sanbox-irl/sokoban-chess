use super::{ComponentBounds, InspectorParameters};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct GridObject {
    grid_type: GridType,
}

impl GridObject {
    pub fn grid_type(&self) -> GridType {
        self.grid_type
    }
}

impl ComponentBounds for GridObject {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_grid_type) =
            super::imgui_system::typed_enum_selection(ip.ui, &self.grid_type, ip.uid)
        {
            self.grid_type = new_grid_type;
        }
    }
}

#[derive(
    Copy,
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    strum_macros::EnumCount,
    typename::TypeName,
)]
pub enum GridType {
    Pushable,
    Blockable,
    NonInteractable,
}

impl Default for GridType {
    fn default() -> GridType {
        GridType::Blockable
    }
}
