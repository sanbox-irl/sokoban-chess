use super::{ComponentBounds, InspectorParameters};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, typename::TypeName, Hash)]
pub struct PrefabMarker {
    pub id: Uuid,
}

impl ComponentBounds for PrefabMarker {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some((name, _)) = &ip.prefabs.get(&self.id).unwrap().name {
            ip.ui.text(imgui::im_str!("Original Prefab: {}", name.name));
        } else {
            ip.ui.text(imgui::im_str!("Original Prefab: {}", self.id));
        }
    }
}
