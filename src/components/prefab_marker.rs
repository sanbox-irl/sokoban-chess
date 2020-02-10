use super::{ComponentBounds, InspectorParameters};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, typename::TypeName, Hash)]
pub struct PrefabMarker {
    pub id: Uuid,
}

impl ComponentBounds for PrefabMarker {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(serialized_name) = &ip.prefabs.get(&self.id).unwrap().name {
            ip.ui.text(imgui::im_str!(
                "Original Prefab: {}",
                serialized_name.inner.name
            ));
        } else {
            ip.ui.text(imgui::im_str!("Original Prefab: {}", self.id));
        }
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .prefab_marker
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.prefab_marker = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.prefab_marker = None;
    }
}
