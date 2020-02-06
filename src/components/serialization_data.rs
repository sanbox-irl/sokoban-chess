use super::{component_serialization::TilemapSerialized, *};
use imgui::*;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, typename::TypeName)]
pub struct SerializationData {
    pub id: Uuid,
}

impl Clone for SerializationData {
    fn clone(&self) -> Self {
        SerializationData::default()
    }
}

impl SerializationData {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub fn with_id(id: Uuid) -> Self {
        Self { id }
    }

    pub fn entity_inspector_serde(
        &self,
        ui: &mut Ui<'_>,
        entity_id: &Entity,
        component_database: &ComponentDatabase,
    ) -> Result<Option<ImGuiSerializationDataCommand>, failure::Error> {
        // We don't show any other prefab stuff...
        if component_database.prefab_markers.get(entity_id).is_some() {
            self.serialization_option(
                ui,
                entity_id,
                &component_database.prefab_markers,
                |se, c| se.prefab_marker = c.fast_serialize(),
                |se| se.prefab_marker = None,
            )?;
            return Ok(None);
        }

        macro_rules! serialization_option_quick {
            ( $( [$x:ident, $y: ident] ),* ) => {
                $(
                    self.serialization_option(
                        ui,
                        entity_id,
                        &component_database.$x,
                        |se, c| se.$y = c.fast_serialize(),
                        |se| se.$y = None,
                    )?;

                )*
            };
        }

        // @update_components
        serialization_option_quick!(
            [names, name],
            [players, player],
            [transforms, transform],
            [grid_objects, grid_object],
            [scene_switchers, scene_switcher]
        );

        self.serialization_option(
            ui,
            entity_id,
            &component_database.graph_nodes,
            |se, c| {
                se.graph_node = Some({
                    let mut clone: super::GraphNode = c.inner().clone();
                    if let Some(children) = clone.children.as_mut() {
                        for child in children.iter_mut() {
                            child.serialize(&component_database.serialization_data);
                        }
                    }
                    ((clone, c.is_active))
                });
            },
            |se| se.follow = None,
        )?;

        serialization_option_quick!(
            [velocities, velocity],
            [sprites, sprite],
            [sound_sources, sound_source],
            [bounding_boxes, bounding_box],
            [draw_rectangles, draw_rectangle]
        );

        self.serialization_option(
            ui,
            entity_id,
            &component_database.tilemaps,
            |se, c| {
                se.tilemap = {
                    let o: &tilemap::Tilemap = c.inner();

                    TilemapSerialized::from_tilemap(o.clone(), &se.id)
                        .map_err(|e| error!("Error Serializing Tiles in Tilemap. Warning: our data might not be saved! {}", e))
                        .ok()
                        .and_then(|ts| Some((ts, c.is_active)))
                };
            },
            |se| se.tilemap = None,
        )?;
        self.serialization_option(
            ui,
            entity_id,
            &component_database.text_sources,
            |se, c| se.text_source = Some((c.inner().clone().into(), c.is_active)),
            |se| se.text_source = None,
        )?;
        self.serialization_option(
            ui,
            entity_id,
            &component_database.follows,
            |se, c| {
                se.follow = Some({
                    let mut clone: super::Follow = c.inner().clone();
                    clone
                        .target
                        .serialize(&component_database.serialization_data);
                    ((clone, c.is_active))
                });
            },
            |se| se.follow = None,
        )?;

        self.serialization_option(
            ui,
            entity_id,
            &component_database.conversant_npcs,
            |se, c| {
                se.conversant_npc = Some({
                    let mut clone: super::ConversantNPC = c.inner().clone();
                    clone
                        .conversation_partner
                        .serialize(&component_database.serialization_data);
                    ((clone, c.is_active))
                });
            },
            |se| se.conversant_npc = None,
        )?;

        ui.spacing();

        let mut sc = None;

        // REVERT SAVE
        let mut cursor_pos = 0.0;
        let (this_size, pressed) =
            imgui_system::sized_button_padding(ui, im_str!("Revert"), Vec2::ZERO);
        if pressed {
            match serialization_util::entities::load_entity(self) {
                Ok(se_option) => match se_option {
                    Some(serialized_entity) => {
                        sc = Some(ImGuiSerializationDataCommand::Revert(serialized_entity))
                    }
                    None => error!(
                        "Couldn't find a serialized entity to revert to! Are you sure this is serialized?"
                    ),
                },
                Err(e) => error!("Error reading serialization file {}", e),
            }
        }

        // OVERWRITE
        cursor_pos += this_size.x + 12.0;
        ui.same_line(cursor_pos);
        let (_, pressed) = imgui_system::sized_button_padding(ui, im_str!("Overwrite"), Vec2::ZERO);
        if pressed {
            sc = Some(ImGuiSerializationDataCommand::Overwrite);
        }

        Ok(sc)
    }

    fn serialization_option<T: ComponentBounds + typename::TypeName + Clone, F1, F2>(
        &self,
        ui: &mut Ui<'_>,
        entity_id: &Entity,
        component_list: &ComponentList<T>,
        serialization_lambda: F1,
        deserialization_lambda: F2,
    ) -> Result<(), failure::Error>
    where
        F1: Fn(&mut SerializedEntity, &Component<T>),
        F2: Fn(&mut SerializedEntity),
    {
        // NAME
        let name = imgui_system::typed_text_ui::<T>();
        if let Some(serialization_option_sub_menu) = ui.begin_menu(
            &ImString::new(name),
            component_list.get(entity_id).is_some(),
        ) {
            if let Some(component) = component_list.get(entity_id) {
                // SERIALIZE
                if MenuItem::new(im_str!("Serialize")).build(ui) {
                    let serialized_entity = serialization_util::entities::load_entity(self)?;

                    if let Some(mut serialized_entity) = serialized_entity {
                        serialization_lambda(&mut serialized_entity, component);
                        serialization_util::entities::serialize_entity(serialized_entity)?;
                    } else {
                        error!(
                            "Couldn't find a Serialized Entity for {}. Check the YAML?",
                            entity_id
                        );
                    }
                }

                // DESERIALIZE
                if MenuItem::new(im_str!("Deserialize")).build(ui) {
                    let serialized_entity = serialization_util::entities::load_entity(self)?;

                    if let Some(mut serialized_entity) = serialized_entity {
                        deserialization_lambda(&mut serialized_entity);
                        serialization_util::entities::serialize_entity(serialized_entity)?;
                    } else {
                        error!(
                            "Couldn't find a Serialized Entity for {}. Check the YAML?",
                            entity_id
                        );
                    }
                }
            }
            serialization_option_sub_menu.end(ui);
        }

        Ok(())
    }

    pub fn edit_serialized_entity<F>(
        serialized_data_list: &ComponentList<SerializationData>,
        entity: &Entity,
        f: F,
    ) -> Result<(), failure::Error>
    where
        F: Fn(&mut SerializedEntity),
    {
        if let Some(serialized_data) = serialized_data_list.get(entity) {
            let serialized_entity =
                serialization_util::entities::load_entity(serialized_data.inner())?;

            if let Some(mut serialized_entity) = serialized_entity {
                f(&mut serialized_entity);
                serialization_util::entities::serialize_entity(serialized_entity)?;
            };
        }

        Ok(())
    }
}

impl Default for SerializationData {
    fn default() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

impl ComponentBounds for SerializationData {
    fn entity_inspector(&mut self, _ip: InspectorParameters<'_, '_>) {}
}

pub enum ImGuiSerializationDataCommand {
    Revert(SerializedEntity),
    Overwrite,
}
