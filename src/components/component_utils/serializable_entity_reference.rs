use super::{imgui_system, ComponentList, Entity, InspectorParameters, SerializationData};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Hash)]
pub struct SerializableEntityReference {
    #[serde(skip)]
    pub target: Option<Entity>,
    target_serialized_id: Option<uuid::Uuid>,
}

impl SerializableEntityReference {
    pub fn new(entity_id: Entity) -> Self {
        Self {
            target: Some(entity_id),
            target_serialized_id: None,
        }
    }

    pub fn blank() -> Self {
        SerializableEntityReference {
            target: None,
            target_serialized_id: None,
        }
    }

    pub fn inspect(&mut self, label: &str, ip: &InspectorParameters<'_, '_>) {
        if let Some(new_target) = imgui_system::select_entity_option(
            label,
            &self.target,
            ip.uid,
            ip.ui,
            ip.entities,
            ip.entity_names,
        ) {
            self.target = new_target;
        }
    }

    pub fn serialize(&mut self, serialized_list: &ComponentList<SerializationData>) {
        if let Some(target) = &self.target {
            if let Some(sd) = serialized_list.get(target) {
                self.target_serialized_id = Some(sd.inner().id.clone());
            } else {
                error!("Reference to {:?} is being serialized, but it is not serialized. We will outlive it and follow nothing on deserialization!", target);
            }
        }
    }

    pub fn deserialize(&mut self, serialized_data: &ComponentList<SerializationData>) {
        if let Some(tsi) = &self.target_serialized_id {
            self.target = serialized_data
                .iter()
                .find(|sd| &sd.inner().id == tsi)
                .map(|i| i.entity_id);

            if self.target.is_none() {
                error!(
                    "We didn't find a target on a serialized entity reference. Did its target have a SerializedData?"
                );
                self.target_serialized_id = None;
            }
        }
    }

    pub fn into_reference(
        maybe_entity: Option<Entity>,
        serialized_data: &ComponentList<SerializationData>,
    ) -> Self {
        if let Some(entity) = maybe_entity {
            SerializableEntityReference {
                target_serialized_id: serialized_data.get(&entity).map(|sd| sd.inner().id),
                target: Some(entity),
            }
        } else {
            SerializableEntityReference {
                target: None,
                target_serialized_id: None,
            }
        }
    }
}
