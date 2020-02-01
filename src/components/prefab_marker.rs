use super::{ComponentBounds, InspectorParameters};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, typename::TypeName, Hash)]
pub struct PrefabMarker {
    pub id: Uuid,
}

impl ComponentBounds for PrefabMarker {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some((name, _)) = &ip.prefabs.get(&self.id).unwrap().name {
            ip.ui.label_text(
                &imgui::im_str!("Prefab##{}", ip.uid),
                &imgui::im_str!("{}", &name.name),
            );
        }
    }
}
