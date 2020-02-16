use super::{serialization_util, ComponentBounds, InspectorParameters, SerializedEntity};
use imgui::*;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, typename::TypeName)]
pub struct SerializationMarker {
    pub id: Uuid,
    #[serde(skip)]
    last_save_data_read: Option<(Instant, SerializedEntity)>,
}

impl Clone for SerializationMarker {
    fn clone(&self) -> Self {
        SerializationMarker::default()
    }
}

impl SerializationMarker {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            last_save_data_read: None,
        }
    }

    /// This is a cache of our Serialized Data. We'll try to get one, but
    /// our serialization data might have been destroyed. If a value is retreived,
    /// it is no more than 5 seconds old.
    pub fn cached_serialized_entity(&mut self) -> Option<&SerializedEntity> {
        if self.last_save_data_read.is_none() {
            self.imgui_serialization();
        }
        self.last_save_data_read.as_ref().map(|s| &s.1)
    }

    pub fn entity_inspector_results(&mut self, ip: InspectorParameters<'_, '_>) -> bool {
        let reload_se = self
            .last_save_data_read
            .as_ref()
            .map_or(true, |(last_date, _)| {
                let time_since: Duration = Instant::now() - *last_date;
                time_since.as_secs() > 5
            });

        if reload_se {
            self.imgui_serialization();
        }

        let mut serialize_entity = false;

        if let Some((last_read, serialized_entity)) = &mut self.last_save_data_read {
            ip.ui.text(im_str!("Entity UUID {}", serialized_entity.id));

            if ip
                .ui
                .button(&im_str!("Copy UUID to Clipboard##{}", ip.uid), [0.0, 0.0])
            {
                ip.ui
                    .set_clipboard_text(&im_str!("{}", serialized_entity.id));
            }
            ip.ui.same_line(0.0);

            if ip
                .ui
                .button(&im_str!("Log Serialized Entity##{}", ip.uid), [0.0, 0.0])
            {
                info!(
                    "Loaded in {}s ago",
                    (Instant::now() - *last_read).as_secs_f32()
                );
                serialized_entity.log_to_console();
            }
        } else {
            if ip
                .ui
                .button(im_str!("Begin Serializing Entity"), [-1.0, 0.0])
            {
                serialize_entity = true;
            }
        }

        if ip.is_open == false {
            self.last_save_data_read = None;
        }

        serialize_entity
    }

    fn imgui_serialization(&mut self) {
        match serialization_util::entities::load_committed_entity(self) {
            Ok(maybe_serialized_entity) => {
                self.last_save_data_read = maybe_serialized_entity.map(|se| (Instant::now(), se))
            }
            Err(e) => {
                error!("Couldn't deserialize entity {}! {}", self.id, e);
            }
        }
    }
}

impl Default for SerializationMarker {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            last_save_data_read: None,
        }
    }
}

impl ComponentBounds for SerializationMarker {
    fn entity_inspector(&mut self, _: InspectorParameters<'_, '_>) {
        unimplemented!();
    }

    fn is_serialized(&self, _: &SerializedEntity, _: bool) -> bool {
        unimplemented!()
    }

    fn commit_to_scene(
        &self,
        _: &mut super::SerializedEntity,
        _: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        unimplemented!();
    }

    fn uncommit_to_scene(&self, _: &mut SerializedEntity) {
        unimplemented!();
    }
}

pub struct ImGuiSerializationDataCommand {
    pub id: uuid::Uuid,
    pub serialization_type: ImGuiSerializationDataType,
}

pub enum ImGuiSerializationDataType {
    Revert,
    Overwrite,
}
