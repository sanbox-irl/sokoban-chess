use super::*;

#[derive(Default)]
pub struct ComponentDatabase {
    pub names: ComponentList<Name>,
    pub prefab_markers: ComponentList<PrefabMarker>,
    pub transforms: ComponentList<Transform>,
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
    pub serialization_data: ComponentList<SerializationData>,
    size: usize,
}

impl ComponentDatabase {
    pub fn new(
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        marker_map: &mut std::collections::HashMap<Marker, Entity>,
    ) -> Result<ComponentDatabase, failure::Error> {
        // CFG if
        if update_serialization::UPDATE_COMPONENT_DATABASE {
            update_serialization::update_component_database()?;
        }
        let saved_entities: Vec<SerializedEntity> =
            serialization_util::entities::load_all_entities()?;

        let mut component_database = ComponentDatabase::default();

        for s_entity in saved_entities {
            let new_entity =
                Ecs::create_entity_raw(&mut component_database, entity_allocator, entities);
            component_database.load_serialized_entity(&new_entity, s_entity, marker_map);
        }

        // Post-Deserialization Work...
        // @update_components
        // @techdebt This probably can turn into a trait on a ComponentBound
        for af in component_database.follows.iter_mut() {
            af.inner_mut()
                .target
                .deserialize(&component_database.serialization_data);
        }

        for conversant_npc in component_database.conversant_npcs.iter_mut() {
            let conversant = conversant_npc.inner_mut();

            conversant
                .conversation_partner
                .deserialize(&component_database.serialization_data);
        }

        for graph_node_c in component_database.graph_nodes.iter_mut() {
            let graph_node: &mut GraphNode = graph_node_c.inner_mut();

            if let Some(children) = &mut graph_node.children {
                for child in children.iter_mut() {
                    child.deserialize(&component_database.serialization_data);
                }
            }
        }

        info!("✔ Loaded Serialized Entities");
        Ok(component_database)
    }

    pub fn register_entity(&mut self, entity: Entity) {
        let index = entity.index();
        if index == self.size {
            self.foreach_component_list(|list| list.expand_list());
            self.size = index + 1;
        }
    }

    pub fn deregister_entity(&mut self, entity: &Entity, maintain_serialization: bool) {
        // Save the Serialization Component Here
        let serialized_data = if maintain_serialization {
            self.serialization_data.get(entity).cloned()
        } else {
            None
        };

        self.foreach_component_list(|list| {
            list.unset(entity);
        });

        // If we've got, resave
        // @techdebt
        if let Some(sd) = serialized_data {
            self.serialization_data.set(entity, sd);
        }
    }

    pub fn clone_components(
        &mut self,
        original: &Entity,
        new_entity: &Entity,
        singleton_database: &SingletonDatabase,
    ) {
        fn clone_list_entry<T: ComponentBounds + Clone>(
            component_list: &mut ComponentList<T>,
            original: &Entity,
            new_entity: &Entity,
        ) {
            if component_list.get(original).is_some() {
                let new_component = component_list.get(original).unwrap().inner().clone();
                component_list.set(new_entity, Component::new(new_entity, new_component));
            }
        }

        // @update_components
        clone_list_entry(&mut self.names, original, new_entity);
        clone_list_entry(&mut self.prefab_markers, original, new_entity);
        clone_list_entry(&mut self.transforms, original, new_entity);
        clone_list_entry(&mut self.graph_nodes, original, new_entity);
        clone_list_entry(&mut self.sprites, original, new_entity);
        clone_list_entry(&mut self.sound_sources, original, new_entity);
        clone_list_entry(&mut self.bounding_boxes, original, new_entity);
        clone_list_entry(&mut self.draw_rectangles, original, new_entity);
        clone_list_entry(&mut self.tilemaps, original, new_entity);
        clone_list_entry(&mut self.text_sources, original, new_entity);
        clone_list_entry(&mut self.transforms, original, new_entity);
        clone_list_entry(&mut self.follows, original, new_entity);
        clone_list_entry(&mut self.conversant_npcs, original, new_entity);

        // Special handling for the SerializedData...
        if self.serialization_data.get(original).is_some() {
            let new_component = SerializationData::new();
            self.serialization_data
                .set(new_entity, Component::new(new_entity, new_component));
            super::serialization_util::entities::serialize_entity_full(
                new_entity,
                self,
                singleton_database,
            );
        }
    }

    pub fn foreach_component_list(&mut self, mut f: impl FnMut(&mut dyn ComponentListBounds)) {
        // @update_components
        f(&mut self.prefab_markers);
        f(&mut self.names);
        f(&mut self.transforms);
        f(&mut self.graph_nodes);
        f(&mut self.velocities);
        f(&mut self.sprites);
        f(&mut self.sound_sources);
        f(&mut self.bounding_boxes);
        f(&mut self.draw_rectangles);
        f(&mut self.tilemaps);
        f(&mut self.text_sources);
        f(&mut self.follows);
        f(&mut self.conversant_npcs);
        f(&mut self.serialization_data);
    }

    pub fn load_serialized_entity(
        &mut self,
        entity: &Entity,
        serialized_entity: SerializedEntity,
        marker_map: &mut std::collections::HashMap<Marker, Entity>,
    ) {
        // Helper macro
        macro_rules! transfer_serialized_components {
            ($component_name: ident, $component_database_name: ident) => {
                if let Some((inner, is_active)) = $component_name {
                    self.$component_database_name.set(
                        &entity,
                        Component::with_active(entity, inner.into(), is_active),
                    );
                }
            };
        }

        let SerializedEntity {
            bounding_box,
            conversant_npc,
            draw_rectangle,
            follow,
            id: _id,
            marker,
            name,
            prefab_marker,
            sound_source,
            sprite,
            text_source,
            tilemap,
            transform,
            velocity,
            graph_node,
        } = serialized_entity;

        // Make a serialization data thingee on it...
        self.serialization_data.set(
            &entity,
            Component::new(
                &entity,
                SerializationData::with_id(serialized_entity.id.clone()),
            ),
        );

        // @update_components
        transfer_serialized_components!(prefab_marker, prefab_markers);
        transfer_serialized_components!(name, names);
        transfer_serialized_components!(transform, transforms);
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
        if let Some((serialized_tilemap, is_active)) = tilemap {
            let tiles: Vec<Option<Tile>> =
                serialization_util::tilemaps::load_tiles(&serialized_tilemap.tiles)
                    .map_err(|e| {
                        error!(
                            "Couldn't retrieve tilemaps for {}. Error: {}",
                            &serialized_tilemap.tiles.relative_path, e
                        )
                    })
                    .ok()
                    .unwrap_or_default();

            let tilemap: tilemap::Tilemap = serialized_tilemap.to_tilemap(tiles);

            self.tilemaps
                .set(entity, Component::with_active(entity, tilemap, is_active));
        }

        // Singleton Components
        if let Some(singleton_marker) = marker {
            marker_map.insert(singleton_marker, *entity);
        }
    }
}
