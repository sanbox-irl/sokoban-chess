use super::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ComponentDatabase {
    pub names: ComponentList<Name>,
    pub prefab_markers: ComponentList<PrefabMarker>,
    pub transforms: ComponentList<Transform>,
    pub players: ComponentList<Player>,
    pub grid_objects: ComponentList<GridObject>,
    pub graph_nodes: ComponentList<GraphNode>,
    pub velocities: ComponentList<Velocity>,
    pub sprites: ComponentList<Sprite>,
    pub sound_sources: ComponentList<SoundSource>,
    pub bounding_boxes: ComponentList<physics_components::BoundingBox>,
    pub draw_rectangles: ComponentList<DrawRectangle>,
    pub tilemaps: ComponentList<tilemap::Tilemap>,
    pub text_sources: ComponentList<TextSource>,
    pub follows: ComponentList<Follow>,
    pub conversant_npcs: ComponentList<ConversantNPC>,
    pub scene_switchers: ComponentList<SceneSwitcher>,
    pub serialization_data: ComponentList<SerializationMarker>,
    size: usize,
}

impl ComponentDatabase {
    pub fn new(
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        marker_map: &mut std::collections::HashMap<Marker, Entity>,
        prefabs: &PrefabMap,
    ) -> Result<ComponentDatabase, failure::Error> {
        // CFG if
        if update_serialization::UPDATE_COMPONENT_DATABASE {
            update_serialization::update_component_database()?;
        }
        let saved_entities: HashMap<Uuid, SerializedEntity> =
            serialization_util::entities::load_all_entities()?;

        let mut component_database = ComponentDatabase::default();

        for (_, s_entity) in saved_entities.into_iter() {
            let new_entity =
                Ecs::create_entity_raw(&mut component_database, entity_allocator, entities);

            component_database.load_serialized_entity(
                &new_entity,
                s_entity,
                entity_allocator,
                entities,
                marker_map,
                prefabs,
            );
        }

        // Post-Deserialization Work...
        // @update_components exceptions
        // @techdebt This probably can turn into a trait on a ComponentBound
        // and probably some unsafe to pass borrow checkrs on serialization_data.
        for af in component_database.follows.iter_mut() {
            af.inner_mut()
                .target
                .serialized_refs_to_entity_id(&component_database.serialization_data);
        }

        for conversant_npc in component_database.conversant_npcs.iter_mut() {
            let conversant = conversant_npc.inner_mut();

            conversant
                .conversation_partner
                .serialized_refs_to_entity_id(&component_database.serialization_data);
        }

        for graph_node_c in component_database.graph_nodes.iter_mut() {
            let graph_node: &mut GraphNode = graph_node_c.inner_mut();

            if let Some(children) = &mut graph_node.children {
                for child in children.iter_mut() {
                    child.serialized_refs_to_entity_id(&component_database.serialization_data);
                }
            }
        }

        Ok(component_database)
    }

    pub fn register_entity(&mut self, entity: Entity) {
        let index = entity.index();
        if index == self.size {
            self.foreach_component_list_mut(NonInspectableEntities::all(), |list| {
                list.expand_list()
            });
            self.size = index + 1;
        }
    }

    pub fn deregister_entity(&mut self, entity: &Entity) {
        self.foreach_component_list_mut(NonInspectableEntities::all(), |list| {
            list.unset(entity);
        });
    }

    pub fn clone_components(&mut self, original: &Entity, new_entity: &Entity) {
        self.foreach_component_list_mut(NonInspectableEntities::all(), |component_list| {
            component_list.clone_entity(original, new_entity);
        });

        // @update_components exceptions
        if let Some(transformc_c) = self.transforms.get_mut(new_entity) {
            scene_graph::add_to_scene_graph(transformc_c, &self.serialization_data)
        }
    }

    // @update_components
    /// This loops over every component, including the non-inspectable ones.
    pub fn foreach_component_list_mut(
        &mut self,
        non_inspectable_entities: NonInspectableEntities,
        mut f: impl FnMut(&mut dyn ComponentListBounds),
    ) {
        if non_inspectable_entities.contains(NonInspectableEntities::NAME) {
            f(&mut self.names);
        }

        self.foreach_component_list_inspectable_mut(&mut f);
        if non_inspectable_entities.contains(NonInspectableEntities::GRAPH_NODE) {
            f(&mut self.graph_nodes);
        }

        if non_inspectable_entities.contains(NonInspectableEntities::PREFAB) {
            f(&mut self.prefab_markers);
        }

        if non_inspectable_entities.contains(NonInspectableEntities::SERIALIZATION) {
            f(&mut self.serialization_data);
        }
    }

    /// This loops over every component except for the following:
    /// - Name
    /// - PrefabMarker
    /// - SerializationMarker
    /// - GraphNode
    /// Use `foreach_component_list` to iterate over all.
    pub fn foreach_component_list_inspectable_mut(
        &mut self,
        f: &mut impl FnMut(&mut dyn ComponentListBounds),
    ) {
        f(&mut self.transforms);
        f(&mut self.grid_objects);
        f(&mut self.players);
        f(&mut self.velocities);
        f(&mut self.sprites);
        f(&mut self.sound_sources);
        f(&mut self.bounding_boxes);
        f(&mut self.draw_rectangles);
        f(&mut self.tilemaps);
        f(&mut self.scene_switchers);
        f(&mut self.text_sources);
        f(&mut self.follows);
        f(&mut self.conversant_npcs);
    }

    // @update_components
    /// This loops over every component, including the non-inspectable ones.
    pub fn foreach_component_list(
        &self,
        non_inspectable_entities: NonInspectableEntities,
        mut f: impl FnMut(&dyn ComponentListBounds),
    ) {
        if non_inspectable_entities.contains(NonInspectableEntities::NAME) {
            f(&self.names);
        }

        self.foreach_component_list_inspectable(&mut f);

        if non_inspectable_entities.contains(NonInspectableEntities::GRAPH_NODE) {
            f(&self.graph_nodes);
        }
        if non_inspectable_entities.contains(NonInspectableEntities::PREFAB) {
            f(&self.prefab_markers);
        }

        if non_inspectable_entities.contains(NonInspectableEntities::SERIALIZATION) {
            f(&self.serialization_data);
        }
    }

    /// This loops over every component except for the following:
    /// - Name
    /// - PrefabMarker
    /// - SerializationMarker
    /// - GraphNode
    /// Use `foreach_component_list` to iterate over all.
    pub fn foreach_component_list_inspectable(&self, f: &mut impl FnMut(&dyn ComponentListBounds)) {
        f(&self.transforms);
        f(&self.grid_objects);
        f(&self.players);
        f(&self.graph_nodes);
        f(&self.velocities);
        f(&self.sprites);
        f(&self.sound_sources);
        f(&self.bounding_boxes);
        f(&self.draw_rectangles);
        f(&self.tilemaps);
        f(&self.scene_switchers);
        f(&self.text_sources);
        f(&self.follows);
        f(&self.conversant_npcs);
    }

    /// We can load anything using this function. The key thing to note here,
    /// however, is that this adds a SerializationData marker to whatever is being
    /// loaded. Ie -- if you load something with this function, it is now serialized.
    pub fn load_serialized_entity(
        &mut self,
        entity: &Entity,
        serialized_entity: SerializedEntity,
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        marker_map: &mut std::collections::HashMap<Marker, Entity>,
        prefabs: &PrefabMap,
    ) {
        // Make a serialization data thingee on it...
        self.serialization_data.set_component(
            &entity,
            SerializationMarker::new(serialized_entity.id.clone()),
        );

        // If it's got a prefab, load the prefab. Otherwise,
        // load it like a normal serialized entity:
        if let Some(serialized_prefab_marker) = &serialized_entity.prefab_marker {
            // Base Prefab
            self.load_serialized_prefab(
                entity,
                &serialized_prefab_marker.inner.main_id(),
                entity_allocator,
                entities,
                prefabs,
            );

            // Overrides
            self.load_serialized_entity_into_database(entity, serialized_entity);
        } else {
            // Singleton Components
            if let Some(singleton_marker) = serialized_entity.marker {
                marker_map.insert(singleton_marker, *entity);
            }

            self.load_serialized_entity_into_database(entity, serialized_entity);
        }
    }

    /// This function loads a prefab directly. Note though, it will not make the resulting
    /// Scene Entity be serialized. To do that, please use `load_serialized_entity`, which
    /// will load the prefab and keep it serialized.
    ///
    /// This function should be used by editor code to instantiate a prefab!
    pub fn load_serialized_prefab(
        &mut self,
        entity_to_load_into: &Entity,
        prefab_id: &Uuid,
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        prefabs: &PrefabMap,
    ) {
        if let Some(prefab) = prefabs.get(&prefab_id) {
            // Load the Main
            let main = prefab
                .members
                .iter()
                .find(|p| p.id == prefab.root_id())
                .unwrap()
                .clone();

            self.load_serialized_entity_into_database(entity_to_load_into, main);
            self.prefab_markers.set_component(
                entity_to_load_into,
                PrefabMarker::new_main(prefab.root_id()),
            );

            let new_entity = Ecs::create_entity_raw(self, entity_allocator, entities);

            // Load the Sub IDs:
            compile_error!("We need to load the sub ids here too! That means reworking a lot of this function...");
        } else {
            error!(
                "Prefab of ID {} does not exist, but we tried to load it into entity {}. We cannot complete this operation.",
                prefab_id,
                Name::get_name_quick(&self.names, entity_to_load_into)
            );
        }
    }

    /// This directly loads a serialized entity into the Ecs. Be careful with this function,
    /// as it does only exactly that. Remember that SerializedEntities do not have a SerializedData
    /// component on them -- we make that for them in `load_serialized_entity`. Maybe we shouldn't do that...

    /// Generally, prefer `load_serialized_entity` or `instantiate_prefab` over this.
    pub fn load_serialized_entity_into_database(
        &mut self,
        entity: &Entity,
        serialized_entity: SerializedEntity,
    ) {
        // @update_components
        let SerializedEntity {
            bounding_box,
            conversant_npc,
            draw_rectangle,
            follow,
            id: _id,
            marker: _marker, // we handle this in `load_serialized_entity`
            name,
            scene_switcher,
            prefab_marker,
            sound_source,
            sprite,
            text_source,
            tilemap,
            transform,
            velocity,
            graph_node,
            player,
            grid_object,
        } = serialized_entity;

        // Helper macro
        macro_rules! transfer_serialized_components {
            ($component_name: ident, $component_database_name: ident) => {
                if let Some(serialized_component) = $component_name {
                    self.$component_database_name.set_component_with_active(
                        &entity,
                        serialized_component.inner,
                        serialized_component.active,
                    );
                }
            };
        }

        // @update_components
        transfer_serialized_components!(prefab_marker, prefab_markers);
        transfer_serialized_components!(name, names);
        transfer_serialized_components!(transform, transforms);
        transfer_serialized_components!(grid_object, grid_objects);
        transfer_serialized_components!(scene_switcher, scene_switchers);
        transfer_serialized_components!(player, players);
        transfer_serialized_components!(graph_node, graph_nodes);
        transfer_serialized_components!(sound_source, sound_sources);
        transfer_serialized_components!(bounding_box, bounding_boxes);
        transfer_serialized_components!(draw_rectangle, draw_rectangles);
        transfer_serialized_components!(text_source, text_sources);
        transfer_serialized_components!(velocity, velocities);
        transfer_serialized_components!(sprite, sprites);
        transfer_serialized_components!(follow, follows);
        transfer_serialized_components!(conversant_npc, conversant_npcs);

        // Tilemap Handling
        if let Some(serialized_component) = tilemap {
            let tiles: Vec<Option<Tile>> =
                serialization_util::tilemaps::load_tiles(&serialized_component.inner.tiles)
                    .map_err(|e| {
                        error!(
                            "Couldn't retrieve tilemaps for {}. Error: {}",
                            &serialized_component.inner.tiles.relative_path, e
                        )
                    })
                    .ok()
                    .unwrap_or_default();

            let tilemap: tilemap::Tilemap = serialized_component.inner.to_tilemap(tiles);

            self.tilemaps
                .set_component_with_active(entity, tilemap, serialized_component.active);
        }
    }
}

impl Default for ComponentDatabase {
    fn default() -> ComponentDatabase {
        ComponentDatabase {
            names: Default::default(),
            prefab_markers: Default::default(),
            transforms: Default::default(),
            players: Default::default(),
            grid_objects: Default::default(),
            graph_nodes: Default::default(),
            velocities: Default::default(),
            sprites: Default::default(),
            sound_sources: Default::default(),
            bounding_boxes: Default::default(),
            draw_rectangles: Default::default(),
            tilemaps: Default::default(),
            text_sources: Default::default(),
            follows: Default::default(),
            conversant_npcs: Default::default(),
            scene_switchers: Default::default(),
            serialization_data: Default::default(),
            size: 0,
        }
    }
}

use bitflags::bitflags;
bitflags! {
    pub struct NonInspectableEntities: u32 {
        const NAME                  =   0b0000_0001;
        const PREFAB                =   0b0000_0010;
        const SERIALIZATION         =   0b0000_0100;
        const GRAPH_NODE            =   0b0000_1000;
    }
}
