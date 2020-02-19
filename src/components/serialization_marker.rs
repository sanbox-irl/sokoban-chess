use super::{
    component_utils::SyncStatus, serialization_util, ComponentBounds, InspectorParameters, SerializedEntity,
};
use imgui::*;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, PartialEq, typename::TypeName)]
pub struct SerializationMarker {
    pub id: Uuid,
    last_save_time: Instant,
    serialized_entity_on_disk: Option<SerializedEntity>,
    force_recache: bool,
}

impl Clone for SerializationMarker {
    fn clone(&self) -> Self {
        SerializationMarker::default()
    }
}

impl SerializationMarker {
    pub fn new(id: Uuid) -> Self {
        let mut me = Self::default();
        me.id = id;
        me
    }

    /// This is a cache of our Serialized Data. We'll try to get one, but
    /// our serialization data might have been destroyed. If a value is retreived,
    /// it is no more than 5 seconds old.
    pub fn cached_serialized_entity(&mut self) -> Option<&SerializedEntity> {
        self.update_cache();
        self.serialized_entity_on_disk.as_ref()
    }

    pub fn entity_inspector_results(&mut self, ip: InspectorParameters<'_, '_>) -> bool {
        self.update_cache();

        let mut serialize_entity = false;

        if let Some(serialized_entity) = &mut self.serialized_entity_on_disk {
            if ip.ui.button(&im_str!("Copy UUID##{}", ip.uid), [0.0, 0.0]) {
                ip.ui.set_clipboard_text(&im_str!("{}", serialized_entity.id));
            }
            ip.ui.same_line(0.0);

            if ip
                .ui
                .button(&im_str!("Log Cached Serialized Entity##{}", ip.uid), [0.0, 0.0])
            {
                println!(
                    "Loaded in {}s ago",
                    (Instant::now() - self.last_save_time).as_secs_f32()
                );
                serialized_entity.log_to_console();
            }
        } else {
            if ip.ui.button(im_str!("Begin Serializing Entity"), [-1.0, 0.0]) {
                serialize_entity = true;
            }
        }

        serialize_entity
    }

    pub fn get_serialization_status(
        &mut self,
        current_serialized_entity: Option<&SerializedEntity>,
    ) -> SyncStatus {
        self.update_cache();

        if let Some(se_on_disk) = &self.serialized_entity_on_disk {
            if let Some(serialized_entity) = current_serialized_entity {
                if se_on_disk == serialized_entity {
                    SyncStatus::Synced
                } else {
                    SyncStatus::OutofSync
                }
            } else {
                // Da fuck is going on here?
                SyncStatus::Headless
            }
        } else {
            SyncStatus::OutofSync
        }
    }

    fn imgui_serialization(&mut self) {
        match serialization_util::entities::load_committed_entity(self) {
            Ok(maybe_serialized_entity) => {
                self.serialized_entity_on_disk = maybe_serialized_entity;
            }
            Err(e) => {
                error!("Couldn't deserialize entity {}! {}", self.id, e);
            }
        }
    }

    fn update_cache(&mut self) {
        let reload_se = {
            let time_since: Duration = Instant::now() - self.last_save_time;
            time_since.as_secs() > 5
        };

        if reload_se || self.force_recache {
            self.force_recache = false;
            self.imgui_serialization();
        }
    }
}

impl Default for SerializationMarker {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            serialized_entity_on_disk: None,
            force_recache: true,
            last_save_time: Instant::now(),
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
